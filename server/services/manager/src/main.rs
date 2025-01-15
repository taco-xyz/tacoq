mod api;
mod config;
mod controller;
mod repo;
mod testing;

use std::net::SocketAddr;

use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use common::brokers::Broker;
use sqlx::PgPool;
use tracing::{info, info_span};

use config::Config;
use repo::{PgRepositoryCore, PgTaskInstanceRepository, PgTaskKindRepository, PgWorkerRepository};

use std::sync::Arc;
use tokio::sync::RwLock;

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
async fn setup_publisher_broker(config: &Config) -> Arc<RwLock<Broker>> {
    // Add the constants here
    Arc::new(RwLock::new(
        Broker::new(
            &config.broker_addr,
            "task_output",
            Some("task_output".to_string()),
            None,
        )
        .await
        .expect("Failed to initialize publisher broker"),
    ))
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
/// * `broker` - The broker
async fn setup_app_state(db_pools: PgPool, broker: Arc<RwLock<Broker>>) -> AppState {
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
async fn setup_app(db_pools: PgPool, broker: Arc<RwLock<Broker>>) -> Router {
    let app_state = setup_app_state(db_pools, broker).await;
    info!("App state created");

    // Create base router with routes and state
    Router::new()
        .merge(api::routes())
        .with_state(app_state)
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let config = Config::new();
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let span = info_span!("manager_startup_real").entered();

    let db_pools = setup_db_pools(&config).await;
    info!("Database connection pools created");

    let broker = setup_publisher_broker(&config).await;
    info!("Broker initialized");

    let app = setup_app(db_pools, broker).await;
    info!("App created");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listener created");

    span.exit();

    axum::serve(listener, app).await.unwrap();
}
