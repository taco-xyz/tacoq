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
use tracing::{info, info_span, warn};

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

/// Creates database connection pools
///
/// # Arguments
///
/// * `config` - The configuration for the database
async fn setup_db_pools(config: &Config) -> PgPool {
    PgPool::connect(&config.db_reader_url).await.unwrap()
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
/// * `broker` - The broker
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
/// * `broker` - The broker
async fn setup_app(db_pools: &PgPool) -> (Router, AppState) {
    let app_state = setup_app_state(db_pools).await;
    info!("App state created");

    // Create base router with routes and state
    let router = Router::new()
        .merge(api::routes())
        .with_state(app_state.clone())
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());

    (router, app_state)
}

async fn initialize_system(
    config: &Config,
) -> Result<
    (
        AppState,
        Server,
        Arc<task::TaskController>,
        Arc<TaskCleanupJob>,
    ),
    Box<dyn std::error::Error>,
> {
    let shutdown = Arc::new(AtomicBool::new(false));

    let db_pools = setup_db_pools(config).await;
    info!("Database connection pools created");

    let new_task_consumer =
        setup_consumer_broker::<Task>(&config.broker_addr, RELAY_QUEUE, shutdown.clone())
            .await
            .expect("Failed to setup task instance consumer");
    info!("Brokers initialized");

    let (app, app_state) = setup_app(&db_pools).await;

    let (task_repo, worker_kind_repo, worker_repo) = create_repositories(&db_pools);

    let task_controller = Arc::new(
        task::TaskController::new(new_task_consumer, worker_repo, worker_kind_repo, task_repo)
            .await?,
    );

    let task_cleanup_job = Arc::new(TaskCleanupJob::new(
        PgTaskRepository::new(PgRepositoryCore::new(db_pools.clone())),
        300, // Every 5 minutes
    ));

    let server = Server::new(app, 3000);

    Ok((app_state, server, task_controller, task_cleanup_job))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = Config::new();
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let span = info_span!("manager_startup").entered();

    let (_, server, task_controller, task_cleanup_job) = initialize_system(&config)
        .await
        .expect("Failed to initialize system");

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Setup shutdown signal handler
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(());
        }
    });

    // Start task cleanup job
    let task_cleanup_handle = tokio::spawn(async move {
        let job = task_cleanup_job.clone();
        job.run().await.expect("Task cleanup job failed");
    });

    // Start task controller
    let task_controller_shutdown = task_controller.clone();
    let task_handle = tokio::spawn(async move {
        let controller = task_controller.clone();
        controller
            .run()
            .await
            .expect("Task input controller failed");
    });

    let server_handle = tokio::spawn(async move {
        server.run(shutdown_rx).await.expect("Server failed");
    });

    span.exit();

    // Wait for shutdown
    tokio::select! {
        _ = task_cleanup_handle => {
            info!("Task cleanup job shutdown");
        },
        _ = task_handle => {
            warn!("Task controller shutdown");
        },
        _ = server_handle => {
            warn!("Server shutdown");
        },
    }

    if let Err(e) = task_controller_shutdown.shutdown().await {
        info!("Failed to shutdown task input controller: {:?}", e);
    }

    info!("Cleanup complete");
}
