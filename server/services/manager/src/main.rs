mod api;
mod config;
mod repo;
mod testing;

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use common::brokers::Broker;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::info;

use config::Config;
use repo::{PgRepositoryCore, PgTaskInstanceRepository, PgTaskKindRepository, PgWorkerRepository};

/// Initializes the logger
async fn setup_logger() {
    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    info!("Logger initialized");
}

/// Represents the shared application state that can be accessed by all routes
///
/// Contains all the repositories used for the application logic and the broker
#[derive(Clone)]
pub struct AppState {
    pub task_repository: PgTaskInstanceRepository,
    pub task_kind_repository: PgTaskKindRepository,
    pub worker_repository: PgWorkerRepository,
    pub broker: Arc<RwLock<Broker>>,
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
async fn setup_broker(config: &Config) -> Broker {
    Broker::new(&config.broker_addr)
        .await
        .expect("Failed to initialize broker")
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
/// * `broker` - The broker
async fn setup_app_state(db_pools: PgPool, broker: Broker) -> AppState {
    // Setup the repositories
    let core = PgRepositoryCore::new(db_pools.clone());
    let task_repository = PgTaskInstanceRepository::new(core.clone());
    let task_kind_repository = PgTaskKindRepository::new(core.clone());
    let worker_repository = PgWorkerRepository::new(core.clone());

    AppState {
        task_repository,
        task_kind_repository,
        worker_repository,
        broker: Arc::new(RwLock::new(broker)),
    }
}

/// Initializes the application router
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
/// * `broker` - The broker
async fn setup_app(db_pools: PgPool, broker: Broker) -> Router {
    let app_state = setup_app_state(db_pools, broker).await;
    info!("App state created");
    Router::new().merge(api::routes()).with_state(app_state)
}

#[tokio::main]
async fn main() {
    let config = Config::new();

    setup_logger().await;

    info!("Starting Galactus");

    let db_pools = setup_db_pools(&config).await;
    info!("Database connection pools created");

    let broker = setup_broker(&config).await;
    info!("Broker initialized");

    let app = setup_app(db_pools, broker).await;
    info!("App created");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
