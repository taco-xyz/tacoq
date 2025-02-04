mod api;
mod config;
mod constants;
mod controller;
mod repo;
mod server;
mod testing;

use common::brokers::{setup_consumer_broker, setup_publisher_broker};
use server::Server;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::oneshot;
use tracing::{info, info_span, warn};

use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use common::{brokers::core::BrokerProducer, models::Task};
use constants::{TASK_INPUT_QUEUE, TASK_OUTPUT_EXCHANGE};
use sqlx::PgPool;

use config::Config;
use controller::task;
use repo::{PgRepositoryCore, PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository};

/// Represents the shared application state that can be accessed by all routes
///
/// Contains all the repositories used for the application logic and the broker
#[derive(Clone)]
pub struct AppState {
    pub task_repository: PgTaskRepository,
    pub worker_kind_repository: PgWorkerKindRepository,
    pub worker_repository: PgWorkerRepository,
    pub broker: Arc<dyn BrokerProducer<Task>>,
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
async fn setup_app_state(db_pools: &PgPool, broker: Arc<dyn BrokerProducer<Task>>) -> AppState {
    let (task_repository, worker_kind_repository, worker_repository) =
        create_repositories(db_pools);

    AppState {
        task_repository,
        worker_kind_repository,
        worker_repository,
        broker,
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
async fn setup_app(db_pools: &PgPool, broker: Arc<dyn BrokerProducer<Task>>) -> (Router, AppState) {
    let app_state = setup_app_state(db_pools, broker).await;
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
) -> Result<(AppState, Server, Arc<task::TaskController>), Box<dyn std::error::Error>> {
    let shutdown = Arc::new(AtomicBool::new(false));

    let db_pools = setup_db_pools(config).await;
    info!("Database connection pools created");

    let publisher_broker =
        setup_publisher_broker::<Task>(&config.broker_addr, TASK_OUTPUT_EXCHANGE)
            .await
            .expect("Failed to setup publisher broker");

    let task_consumer =
        setup_consumer_broker::<Task>(&config.broker_addr, TASK_INPUT_QUEUE, shutdown.clone())
            .await
            .expect("Failed to setup task result consumer");

    let (app, app_state) = setup_app(&db_pools, publisher_broker).await;

    let (task_repo, worker_kind_repo, worker_repo) = create_repositories(&db_pools);

    let task_controller = Arc::new(
        task::TaskController::new(task_consumer, worker_repo, worker_kind_repo, task_repo).await?,
    );

    let server = Server::new(app, 3000);

    Ok((app_state, server, task_controller))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = Config::new();
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let span = info_span!("manager_startup_real").entered();

    let (_, server, task_controller) = initialize_system(&config)
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

    let task_handle = tokio::spawn({
        let controller = task_controller.clone();
        async move {
            controller
                .run()
                .await
                .expect("Task input controller failed");
        }
    });

    let server_handle = tokio::spawn(async move {
        server.run(shutdown_rx).await.expect("Server failed");
    });

    span.exit();

    // Wait for shutdown
    tokio::select! {
        _ = task_handle => {
            warn!("Task controller shutdown");
        },
        _ = server_handle => {
            warn!("Server shutdown");
        },
    }

    // Graceful shutdown
    if let Err(e) = task_controller.shutdown().await {
        info!("Failed to shutdown task input controller: {:?}", e);
    }

    info!("Cleanup complete");
}
