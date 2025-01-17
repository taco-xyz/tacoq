mod api;
mod config;
mod constants;
mod controller;
mod repo;
mod testing;

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use common::brokers::rabbit::{RabbitBrokerCore, TaskInstanceRabbitMQProducer};
use constants::TASK_OUTPUT_EXCHANGE;
use controller::{task_input::TaskInputController, task_result::TaskResultController};
use sqlx::PgPool;
use tokio::sync::oneshot;
use tracing::{info, info_span};

use config::Config;
use repo::{PgRepositoryCore, PgTaskInstanceRepository, PgTaskKindRepository, PgWorkerRepository};

/// Represents the shared application state that can be accessed by all routes
///
/// Contains all the repositories used for the application logic and the broker
#[derive(Clone)]
pub struct AppState {
    pub task_repository: PgTaskInstanceRepository,
    pub task_kind_repository: PgTaskKindRepository,
    pub worker_repository: PgWorkerRepository,
    pub broker: TaskInstanceRabbitMQProducer,
}

impl AppState {
    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.broker.cleanup().await
    }
}

/// Creates database connection pools
///
/// # Arguments
///
/// * `config` - The configuration for the database
async fn setup_db_pools(config: &Config) -> PgPool {
    PgPool::connect(&config.db_reader_url).await.unwrap()
}

/// Initializes the broker
///
/// # Arguments
///
/// * `config` - The configuration for the broker   
async fn setup_publisher_broker(config: &Config) -> TaskInstanceRabbitMQProducer {
    let core = RabbitBrokerCore::new(&config.broker_addr.clone())
        .await
        .expect("Failed to initialize publisher broker");

    TaskInstanceRabbitMQProducer::new(core, TASK_OUTPUT_EXCHANGE)
        .await
        .expect("Failed to initialize publisher broker")
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
/// * `broker` - The broker
async fn setup_app_state(db_pools: &PgPool, broker: TaskInstanceRabbitMQProducer) -> AppState {
    // Setup the repositories
    let core = PgRepositoryCore::new(db_pools.clone());
    let task_repository = PgTaskInstanceRepository::new(core.clone());
    let task_kind_repository = PgTaskKindRepository::new(core.clone());
    let worker_repository = PgWorkerRepository::new(core.clone());

    AppState {
        task_repository,
        task_kind_repository,
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
async fn setup_app(db_pools: &PgPool, broker: TaskInstanceRabbitMQProducer) -> (Router, AppState) {
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

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = Config::new();
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let span = info_span!("manager_startup_real").entered();

    let db_pools = setup_db_pools(&config).await;
    info!("Database connection pools created");

    // Create two separate publisher brokers
    let broker = setup_publisher_broker(&config).await;
    info!("Brokers initialized");

    let (app, mut app_state) = setup_app(&db_pools, broker).await;
    info!("App created");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    // Create task repositories for controllers
    let core = PgRepositoryCore::new(db_pools);
    let task_repo = Arc::new(PgTaskInstanceRepository::new(core));

    // Initialize controllers
    let task_input_controller = TaskInputController::new(&config.broker_addr, task_repo.clone())
        .await
        .expect("Failed to create task input controller");

    let task_result_controller = TaskResultController::new(&config.broker_addr, task_repo.clone())
        .await
        .expect("Failed to create task result controller");

    let is_running = Arc::new(AtomicBool::new(true));
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Clone controllers for the shutdown handler
    let task_input_controller_shutdown = task_input_controller.clone();
    let task_result_controller_shutdown = task_result_controller.clone();

    // Setup shutdown signal handler
    let is_running_signal = is_running.clone();
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await {
            info!("Shutdown signal received");
            is_running_signal.store(false, Ordering::SeqCst);
            let _ = shutdown_tx.send(());
        }
    });

    // Spawn controllers
    let input_handle = tokio::spawn(async move {
        task_input_controller
            .run()
            .await
            .expect("Task input controller failed");
    });

    let result_handle = tokio::spawn(async move {
        task_result_controller
            .run()
            .await
            .expect("Task result controller failed");
    });

    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .unwrap()
    });

    span.exit();

    // Wait for shutdown signal
    tokio::select! {
        _ = input_handle => {},
        _ = result_handle => {},
        _ = server_handle => {},
    }

    // Cleanup
    info!("Starting cleanup");
    if let Err(e) = task_input_controller_shutdown.cleanup().await {
        info!("Error cleaning up task input controller: {}", e);
    }
    if let Err(e) = task_result_controller_shutdown.cleanup().await {
        info!("Error cleaning up task result controller: {}", e);
    }

    if let Err(e) = app_state.cleanup().await {
        info!("Error cleanin up app state: {}", e)
    }

    info!("Cleanup complete");
}
