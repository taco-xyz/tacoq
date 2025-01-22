mod api;
mod config;
mod constants;
mod controller;
mod repo;
mod server;
mod testing;

use common::TaskResult;
use controller::Controllers;
use server::Server;
use std::fmt::Debug;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::oneshot;
use tracing::{info, info_span};

use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use common::{
    brokers::{
        core::BrokerProducer,
        rabbit::{RabbitBrokerCore, RabbitMQConsumer, RabbitMQProducer},
    },
    TaskInstance,
};
use constants::{TASK_INPUT_QUEUE, TASK_OUTPUT_EXCHANGE, TASK_RESULT_QUEUE};
use sqlx::PgPool;

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
    pub broker: Arc<dyn BrokerProducer<TaskInstance>>,
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

/// Initializes a generic producer broker
///
/// # Arguments
///
/// * `config` - The configuration for the broker   
/// * `exchange` - The exchange to publish messages to
async fn setup_publisher_broker<T>(config: &Config, exchange: &str) -> Arc<RabbitMQProducer<T>>
where
    T: Debug,
{
    let core = RabbitBrokerCore::new(&config.broker_addr.clone())
        .await
        .expect("Failed to initialize publisher broker");

    Arc::new(
        RabbitMQProducer::<T>::new(core, exchange)
            .await
            .expect("Failed to initialize publisher broker"),
    )
}

/// Initializes a generic consumer broker
///
/// # Arguments
///
/// * `config` - The configuration for the broker
/// * `queue` - The queue to consume messages from
/// * `is_running` - A flag indicating if the application is running
async fn setup_consumer_broker<T>(
    config: &Config,
    queue: &str,
    is_running: Arc<AtomicBool>,
) -> Arc<RabbitMQConsumer<T>>
where
    T: Debug,
{
    let core = RabbitBrokerCore::new(&config.broker_addr.clone())
        .await
        .expect("Failed to initialize consumer broker");

    Arc::new(
        RabbitMQConsumer::<T>::new(core, queue, is_running)
            .await
            .expect("Failed to initialize consumer broker"),
    )
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
/// * `broker` - The broker
async fn setup_app_state(
    db_pools: &PgPool,
    broker: Arc<dyn BrokerProducer<TaskInstance>>,
) -> AppState {
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
async fn setup_app(
    db_pools: &PgPool,
    broker: Arc<dyn BrokerProducer<TaskInstance>>,
) -> (Router, AppState) {
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
) -> Result<(AppState, Controllers, Server), Box<dyn std::error::Error>> {
    let is_running = Arc::new(AtomicBool::new(true));

    let db_pools = setup_db_pools(config).await;
    info!("Database connection pools created");

    let publisher_broker =
        setup_publisher_broker::<TaskInstance>(config, TASK_OUTPUT_EXCHANGE).await;

    let task_result_consumer =
        setup_consumer_broker::<TaskResult>(config, TASK_RESULT_QUEUE, is_running.clone()).await;

    let task_instance_consumer =
        setup_consumer_broker::<TaskInstance>(config, TASK_INPUT_QUEUE, is_running.clone()).await;
    info!("Brokers initialized");

    let (app, app_state) = setup_app(&db_pools, publisher_broker).await;

    let core = PgRepositoryCore::new(db_pools);
    let task_repo = Arc::new(PgTaskInstanceRepository::new(core));

    let controllers =
        Controllers::new(task_instance_consumer, task_result_consumer, task_repo).await?;

    let server = Server::new(app, 3000);

    Ok((app_state, controllers, server))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = Config::new();
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let span = info_span!("manager_startup_real").entered();

    let (mut app_state, controllers, server) = initialize_system(&config)
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

    let (input_handle, result_handle) = controllers.run().await;
    let server_handle = tokio::spawn(async move {
        server.run(shutdown_rx).await.expect("Server failed");
    });

    span.exit();

    // Wait for shutdown
    tokio::select! {
        _ = input_handle => {},
        _ = result_handle => {},
        _ = server_handle => {},
    }

    // Cleanup
    info!("Starting cleanup");
    controllers.cleanup().await;
    if let Err(e) = app_state.cleanup().await {
        info!("Error cleaning up app state: {}", e);
    }
    info!("Cleanup complete");
}
