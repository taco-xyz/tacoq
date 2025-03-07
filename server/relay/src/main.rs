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

use brokers::setup_consumer_broker;
use server::Server;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::oneshot;
use tracing::{error, info, info_span, warn};

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
fn init_tracing() -> Result<impl Drop, Box<dyn std::error::Error>> {
    Ok(init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?)
}

/// Creates database connection pools
///
/// # Arguments
///
/// * `config` - The configuration for the database
async fn setup_db_pools(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(&config.db_reader_url).await
}

/// Creates all repositories needed for the application
///
/// # Arguments
///
/// * `pool` - The database connection pool
fn create_repositories(
    pool: &PgPool,
) -> (PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository) {
    let core = PgRepositoryCore::new(pool.clone());
    let task_repository = PgTaskRepository::new(core.clone());
    let worker_kind_repository = PgWorkerKindRepository::new(core.clone());
    let worker_repository = PgWorkerRepository::new(core.clone());

    (task_repository, worker_kind_repository, worker_repository)
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
async fn setup_app_state(db_pools: &PgPool) -> AppState {
    let (task_repository, worker_kind_repository, worker_repository) =
        create_repositories(db_pools);

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
    let app_state = setup_app_state(db_pools).await;
    info!("App state created");

    // Create base router with routes and state
    let router = Router::new()
        .merge(api::routes())
        .with_state(app_state)
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());

    router
}

/// Sets up a shutdown signal handler
///
/// Returns a channel receiver that will be notified when shutdown is requested
async fn setup_shutdown_signal() -> oneshot::Receiver<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(());
        }
    });

    shutdown_rx
}

/// Initializes all application components
///
/// # Arguments
///
/// * `config` - The application configuration
async fn initialize_system(config: &Config) -> Result<AppComponents, Box<dyn std::error::Error>> {
    let shutdown = Arc::new(AtomicBool::new(false));

    // Setup database connection
    let db_pools = setup_db_pools(config).await?;
    info!("Database connection pools created");

    // Setup message broker
    let new_task_consumer =
        setup_consumer_broker::<Task>(&config.broker_addr, RELAY_QUEUE, shutdown.clone()).await?;
    info!("Brokers initialized");

    // Setup axum app and state
    let app = setup_app(&db_pools).await;

    // Create repositories
    let (task_repo, worker_kind_repo, worker_repo) = create_repositories(&db_pools);

    // Initialize controller and job
    let task_controller = Arc::new(
        task::TaskController::new(new_task_consumer, worker_repo, worker_kind_repo, task_repo)
            .await?,
    );

    let task_cleanup_job = Arc::new(TaskCleanupJob::new(
        PgTaskRepository::new(PgRepositoryCore::new(db_pools.clone())),
        300, // Every 5 minutes
    ));

    // Create server
    let server = Server::new(app, 3000);

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
    // Start task cleanup job
    let task_cleanup_handle = tokio::spawn(async move {
        if let Err(e) = components.task_cleanup_job.run().await {
            error!("Task cleanup job failed: {:?}", e);
        }
    });

    // Keep a reference for shutdown
    let task_controller_shutdown = components.task_controller.clone();

    // Start task controller
    let task_handle = tokio::spawn(async move {
        if let Err(e) = components.task_controller.run().await {
            error!("Task input controller failed: {:?}", e);
        }
    });

    // Start server
    let server_handle = tokio::spawn(async move {
        if let Err(e) = components.server.run(shutdown_rx).await {
            error!("Server error: {:?}", e);
        }
    });

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
    info!("Starting graceful shutdown");

    if let Err(e) = task_controller.shutdown().await {
        error!("Failed to shutdown task input controller: {:?}", e);
    }

    info!("Cleanup complete");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    let config = Config::new();

    // Setup tracing
    let _guard = init_tracing()?;

    let span = info_span!("manager_startup").entered();

    // Initialize system components
    let components = match initialize_system(&config).await {
        Ok(components) => components,
        Err(e) => {
            error!("Failed to initialize system: {:?}", e);
            return Err(e);
        }
    };

    // Setup shutdown signal
    let shutdown_rx = setup_shutdown_signal().await;

    // Start all background tasks
    let (task_cleanup_handle, task_handle, server_handle, task_controller_shutdown) =
        start_background_tasks(components, shutdown_rx).await;

    span.exit();

    // Wait for any task to complete, which signals shutdown
    tokio::select! {
        _ = task_cleanup_handle => {
            warn!("Task cleanup job shutdown unexpectedly");
        },
        _ = task_handle => {
            warn!("Task controller shutdown unexpectedly");
        },
        _ = server_handle => {
            warn!("Server shutdown unexpectedly");
        },
    }

    // Perform graceful shutdown
    perform_shutdown(task_controller_shutdown).await;

    Ok(())
}
