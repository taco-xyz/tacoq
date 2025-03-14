mod api;
mod brokers;
mod config;
mod constants;
mod controller;
mod jobs;
mod models;
mod repo;
mod server;
mod testing;
// mod traces;

use brokers::setup_consumer_broker;
use init_tracing_opentelemetry::tracing_subscriber_ext::{
    build_level_filter_layer, build_otel_layer,
};
use server::Server;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::oneshot;
use tracing::{debug, error, info, info_span, warn};
use tracing_subscriber::{layer::SubscriberExt, Layer};

use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use constants::RELAY_QUEUE;
use models::Task;
use sqlx::PgPool;

use config::Config;
use controller::task;
use jobs::TaskCleanupJob;
use repo::{PgRepositoryCore, PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository};

/// Represents the shared application state that can be accessed by all routes
///
/// Contains all the repositories used for the application logic and the broker
#[derive(Clone)]
pub struct AppState {
    pub task_repository: PgTaskRepository,
    pub worker_kind_repository: PgWorkerKindRepository,
    pub worker_repository: PgWorkerRepository,
}

/// Application components that need to be started and shut down
struct AppComponents {
    server: Server,
    task_controller: Arc<task::TaskController>,
    task_cleanup_job: Arc<TaskCleanupJob>,
}

/// Initializes the tracing system
/// Initializes the unified tracing system with both local console output and OpenTelemetry
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

/// Creates database connection pools
///
/// # Arguments
///
/// * `config` - The configuration for the database
async fn setup_db_pools(config: &Config) -> Result<PgPool, sqlx::Error> {
    info!(
        db_url_length = config.db_url.len(),
        "Connecting to database"
    );

    let pool = match PgPool::connect(&config.db_url).await {
        Ok(pool) => {
            info!("Successfully connected to database");
            pool
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to database");
            return Err(e);
        }
    };

    // Run migrations on the database
    debug!("Running database migrations");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => {
            info!("Database migrations completed successfully");
            Ok(pool)
        }
        Err(e) => {
            error!(error = %e, "Failed to run database migrations");
            Err(e.into())
        }
    }
}

/// Creates all repositories needed for the application
///
/// # Arguments
///
/// * `pool` - The database connection pool
fn create_repositories(
    pool: &PgPool,
) -> (PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository) {
    debug!("Creating repository core");
    let core = PgRepositoryCore::new(pool.clone());

    debug!("Creating task repository");
    let task_repository = PgTaskRepository::new(core.clone());

    debug!("Creating worker kind repository");
    let worker_kind_repository = PgWorkerKindRepository::new(core.clone());

    debug!("Creating worker repository");
    let worker_repository = PgWorkerRepository::new(core.clone());

    debug!("All repositories created successfully");
    (task_repository, worker_kind_repository, worker_repository)
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
async fn setup_app_state(db_pools: &PgPool) -> AppState {
    debug!("Setting up application state");
    let (task_repository, worker_kind_repository, worker_repository) =
        create_repositories(db_pools);

    info!("Application state initialized successfully");
    AppState {
        task_repository,
        worker_kind_repository,
        worker_repository,
    }
}

/// Initializes the application router
///
/// Also injects it with tracing middleware to create spans across the application
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
async fn setup_app(db_pools: &PgPool) -> Router {
    debug!("Beginning app setup");
    let app_state = setup_app_state(db_pools).await;
    info!("App state created");

    // Create base router with routes and state
    debug!("Creating router with OpenTelemetry layers");
    let router = Router::new()
        .merge(api::routes())
        .with_state(app_state)
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());

    info!("Router setup complete with tracing enabled");
    router
}

/// Sets up a shutdown signal handler
///
/// Returns a channel receiver that will be notified when shutdown is requested
async fn setup_shutdown_signal() -> oneshot::Receiver<()> {
    debug!("Setting up shutdown signal handler");
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        debug!("Waiting for shutdown signal");
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal (Ctrl+C) received");
            if shutdown_tx.send(()).is_err() {
                error!("Failed to send shutdown signal");
            }
        }
    });

    info!("Shutdown signal handler initialized");
    shutdown_rx
}

/// Initializes all application components
///
/// # Arguments
///
/// * `config` - The application configuration
async fn initialize_system(config: &Config) -> Result<AppComponents, Box<dyn std::error::Error>> {
    debug!("Initializing system components");
    let shutdown = Arc::new(AtomicBool::new(false));

    // Setup database connection
    let db_pools = match setup_db_pools(config).await {
        Ok(pools) => pools,
        Err(e) => {
            error!(error = %e, "Database connection setup failed");
            return Err(Box::new(e));
        }
    };
    info!("Database connection pools created");

    // Setup message broker
    debug!(
        broker_url = %config.broker_url,
        queue = %RELAY_QUEUE,
        "Setting up message broker consumer"
    );
    let new_task_consumer = match setup_consumer_broker::<Task>(
        &config.broker_url,
        RELAY_QUEUE,
        shutdown.clone(),
    )
    .await
    {
        Ok(consumer) => {
            info!("Message broker consumer initialized successfully");
            consumer
        }
        Err(e) => {
            error!(
                error = %e,
                broker_url = %config.broker_url,
                queue = %RELAY_QUEUE,
                "Failed to setup message broker consumer"
            );
            return Err(e);
        }
    };

    // Setup axum app and state
    debug!("Setting up web application");
    let app = setup_app(&db_pools).await;

    // Create repositories
    debug!("Creating repositories for components");
    let (task_repo, worker_kind_repo, worker_repo) = create_repositories(&db_pools);

    // Initialize controller and job
    debug!("Creating task controller");
    let task_controller = match task::TaskController::new(
        new_task_consumer,
        worker_repo,
        worker_kind_repo,
        task_repo,
    )
    .await
    {
        Ok(controller) => {
            info!("Task controller initialized successfully");
            Arc::new(controller)
        }
        Err(e) => {
            error!(error = %e, "Failed to initialize task controller");
            return Err(e);
        }
    };

    debug!("Creating task cleanup job with 5-minute interval");
    let task_cleanup_job = Arc::new(TaskCleanupJob::new(
        PgTaskRepository::new(PgRepositoryCore::new(db_pools.clone())),
        300, // Every 5 minutes
    ));
    info!("Task cleanup job created with 300-second interval");

    // Create server
    debug!("Creating HTTP server on port 3000");
    let server = Server::new(app, 3000);
    info!(port = 3000, "HTTP server created");

    info!("All system components initialized successfully");
    Ok(AppComponents {
        server,
        task_controller,
        task_cleanup_job,
    })
}

/// Starts all application background tasks
///
/// Returns handles and a channel to coordinate shutdown
///
/// # Arguments
///
/// * `components` - The application components
async fn start_background_tasks(
    components: AppComponents,
    shutdown_rx: oneshot::Receiver<()>,
) -> (
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
    Arc<task::TaskController>,
) {
    debug!("Starting background tasks");

    // Start task cleanup job
    info!("Starting task cleanup job");
    let task_cleanup_handle = tokio::spawn(async move {
        debug!("Task cleanup job started");
        if let Err(e) = components.task_cleanup_job.run().await {
            error!(error = %e, "Task cleanup job failed");
        } else {
            info!("Task cleanup job completed successfully");
        }
    });

    // Keep a reference for shutdown
    let task_controller_shutdown = components.task_controller.clone();

    // Start task controller
    info!("Starting task controller");
    let task_handle = tokio::spawn(async move {
        debug!("Task controller started");
        if let Err(e) = components.task_controller.run().await {
            error!(error = %e, "Task input controller failed");
        } else {
            info!("Task controller completed successfully");
        }
    });

    // Start server
    info!("Starting HTTP server");
    let server_handle = tokio::spawn(async move {
        debug!("HTTP server started");
        if let Err(e) = components.server.run(shutdown_rx).await {
            error!(error = %e, "Server error");
        } else {
            info!("HTTP server shut down gracefully");
        }
    });

    info!("All background tasks started");
    (
        task_cleanup_handle,
        task_handle,
        server_handle,
        task_controller_shutdown,
    )
}

/// Performs graceful shutdown of all components
///
/// # Arguments
///
/// * `task_controller` - The task controller to shut down
async fn perform_shutdown(task_controller: Arc<task::TaskController>) {
    info!("Starting graceful shutdown procedure");

    debug!("Shutting down task controller");
    match task_controller.shutdown().await {
        Ok(_) => info!("Task controller shut down successfully"),
        Err(e) => error!(error = %e, "Failed to shutdown task input controller"),
    }

    info!("All components shut down, cleanup complete");
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

    // Initialize system components
    debug!("Initializing system components");
    let components = match initialize_system(&config).await {
        Ok(components) => components,
        Err(e) => {
            error!(error = %e, "Failed to initialize system");
            return Err(e);
        }
    };

    // Setup shutdown signal
    debug!("Setting up shutdown signal handler");
    let shutdown_rx = setup_shutdown_signal().await;

    // Start all background tasks
    info!("Starting all background tasks and services");
    let (task_cleanup_handle, task_handle, server_handle, task_controller_shutdown) =
        start_background_tasks(components, shutdown_rx).await;

    info!("Relay service startup complete, now running");
    span.exit();

    // Wait for any task to complete, which signals shutdown
    tokio::select! {
        result = task_cleanup_handle => {
            warn!(task = "task_cleanup", result = ?result, "Task cleanup job shutdown unexpectedly");
        },
        result = task_handle => {
            warn!(task = "task_controller", result = ?result, "Task controller shutdown unexpectedly");
        },
        result = server_handle => {
            warn!(task = "http_server", result = ?result, "Server shutdown unexpectedly");
        },
    }

    // Perform graceful shutdown
    info!("Beginning shutdown sequence");
    perform_shutdown(task_controller_shutdown).await;
    info!("Relay service shut down successfully");

    Ok(())
}
