mod api;
mod brokers;
mod config;
mod constants;
mod controller;
mod jobs;
mod lifecycle;
mod models;
mod repo;
mod server;
mod testing;
// mod traces;

use init_tracing_opentelemetry::tracing_subscriber_ext::{
    build_level_filter_layer, build_otel_layer,
};
use tracing::{debug, error, info, info_span, warn};
use tracing_subscriber::{layer::SubscriberExt, Layer};

use config::Config;

/// Represents the shared application state that can be accessed by all routes
///
/// Contains all the repositories used for the application logic and the broker
#[derive(Clone)]
pub struct AppState {
    pub task_repository: repo::PgTaskRepository,
    pub worker_kind_repository: repo::PgWorkerKindRepository,
    pub worker_repository: repo::PgWorkerRepository,
}

/// Initializes the tracing system
/// Initializes the unified tracing system with both local console output and OpenTelemetry
fn init_tracing() -> Result<impl Drop, Box<dyn std::error::Error>> {
    let logger_text: Box<dyn Layer<_> + Send + Sync + 'static> = if cfg!(debug_assertions) {
        // TODO: check if we need more infmormation in these logs
        // Development environment - human-readable logs with detailed context
        Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_line_number(true)
                .with_file(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_ansi(true),
        )
    } else {
        // Production environment - structured JSON logs for machine processing
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .flatten_event(true) // Better for log aggregation systems
                .with_timer(tracing_subscriber::fmt::time::SystemTime),
        )
    };

    let (layer, guard) = build_otel_layer()?;

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(build_level_filter_layer("")?)
        .with(logger_text);
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(guard)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    debug!("Loading application configuration");
    let config = Config::new();
    info!("Configuration loaded successfully");

    // Setup tracing
    debug!("Initializing tracing system");
    let _guard = match init_tracing() {
        Ok(guard) => {
            info!("Tracing initialized successfully");
            guard
        }
        Err(e) => {
            eprintln!("Failed to initialize tracing: {:?}", e);
            return Err(e);
        }
    };

    let span = info_span!("manager_startup", service = "relay").entered();
    info!("Starting Relay service");

    // Log which services are enabled
    info!(
        task_consumer = config.enable_relay_task_consumer,
        cleanup = config.enable_relay_cleanup,
        api = config.enable_relay_api,
        "Service configuration"
    );

    // If no services are enabled, exit gracefully
    if !config.enable_relay_task_consumer
        && !config.enable_relay_cleanup
        && !config.enable_relay_api
    {
        warn!("No services are enabled, exiting");
        return Ok(());
    }

    // Setup broadcast channel for coordinating shutdown
    let shutdown_signal = lifecycle::setup_shutdown_signal().await;

    // Initialize system components
    debug!("Initializing system components");
    let components = match lifecycle::initialize_system(&config, shutdown_signal.clone()).await {
        Ok(components) => components,
        Err(e) => {
            error!(error = %e, "Failed to initialize system");
            return Err(e);
        }
    };

    // Start all enabled background tasks
    info!("Starting enabled background tasks and services");
    let (handles, task_controller_shutdown) =
        lifecycle::start_background_tasks(components, shutdown_signal.clone()).await;

    if handles.is_empty() {
        warn!("No services were started, exiting");
        return Ok(());
    }

    info!("Relay service startup complete, now running");
    span.exit();

    // Wait for first task to complete or for shutdown signal
    let (result, _, _) = futures::future::select_all(handles).await;
    if let Err(e) = result {
        error!(error = %e, "One of the services failed");
    }

    // Trigger shutdown for all tasks if not already triggered
    let _ = shutdown_signal.send(());

    // Perform graceful shutdown
    info!("Beginning shutdown sequence");
    lifecycle::perform_shutdown(task_controller_shutdown).await;
    info!("Relay service shut down successfully");

    Ok(())
}
