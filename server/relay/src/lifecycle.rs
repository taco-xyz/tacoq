use crate::constants::RELAY_QUEUE;
use crate::jobs::TaskCleanupJob;
use crate::repo::{PgRepositoryCore, TaskRepository};
use crate::server::Server;
use crate::task_event_consumer::{RabbitMQTaskEventConsumer, TaskEventConsumer};
use crate::{api, Config};
use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use backoff::ExponentialBackoffBuilder;
use sqlx::PgPool;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Represents the shared application state that can be accessed by all routes
///
/// Contains all the repositories used for the application logic and the broker
#[derive(Clone)]
pub struct AppState {
    pub task_repository: TaskRepository,

    // Health check variables
    pub repository_core: PgRepositoryCore,
    // Here I wanna pass a rabbitmq channel to verify if the connection is still alive
    // For now I will leave it as is but in the future this will need to be abstracted away
    pub broker_core: Option<Arc<lapin::Channel>>,
}

/// Application components that need to be started and shut down
pub struct AppComponents {
    pub server: Option<Server>,
    pub update_consumer: Option<Arc<RabbitMQTaskEventConsumer>>,
    pub task_cleanup_job: Option<Arc<TaskCleanupJob>>,
}

/// Creates database connection pools
///
/// # Arguments
///
/// * `config` - The configuration for the database
pub async fn setup_db_pools(config: &Config) -> Result<PgPool, sqlx::Error> {
    info!(
        db_url_length = config.db_url.len(),
        "Connecting to database"
    );

    // Configure backoff strategy
    let backoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(10))
        .build();

    // Try to connect with retries
    let pool = match backoff::future::retry(backoff, || async {
        match PgPool::connect(&config.db_url).await {
            Ok(pool) => Ok(pool),
            Err(e) => {
                warn!(error = %e, "Failed to connect to database, retrying...");
                Err(backoff::Error::transient(e))
            }
        }
    })
    .await
    {
        Ok(pool) => {
            info!("Successfully connected to database");
            pool
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to database after retries");
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
fn create_repositories(pool: &PgPool) -> TaskRepository {
    debug!("Creating repository core");
    let core = PgRepositoryCore::new(pool.clone());

    debug!("Creating task repository");
    let task_repository = TaskRepository::new(core.clone());

    debug!("All repositories created successfully");
    task_repository
}

/// Initializes the application state based on the given configuration
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
async fn setup_app_state(db_pools: &PgPool, broker_core: Option<Arc<lapin::Channel>>) -> AppState {
    debug!("Setting up application state");
    let task_repository = create_repositories(db_pools);

    info!("Application state initialized successfully");
    AppState {
        task_repository,
        repository_core: PgRepositoryCore::new(db_pools.clone()),
        broker_core,
    }
}

/// Initializes the application router
///
/// Also injects it with tracing middleware to create spans across the application
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
pub async fn setup_app(db_pools: &PgPool, broker_core: Option<Arc<lapin::Channel>>) -> Router {
    debug!("Beginning app setup");
    let app_state = setup_app_state(db_pools, broker_core).await;
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
/// Returns a channel sender that can be used to notify components of shutdown
pub async fn setup_shutdown_signal() -> broadcast::Sender<()> {
    debug!("Setting up shutdown signal handler");
    let (shutdown_tx, _) = broadcast::channel(1);
    let tx_clone = shutdown_tx.clone();

    tokio::spawn(async move {
        debug!("Waiting for shutdown signal");
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal (Ctrl+C) received");
            let _ = tx_clone.send(());
        }
    });

    info!("Shutdown signal handler initialized");
    shutdown_tx
}

/// Initializes all application components based on configuration
///
/// # Arguments
///
/// * `config` - The application configuration
/// * `shutdown_signal` - Broadcast channel for shutdown coordination
pub async fn initialize_system(
    config: &Config,
    shutdown_signal: broadcast::Sender<()>,
) -> Result<AppComponents, Box<dyn std::error::Error + Send + Sync>> {
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

    // Create repositories
    debug!("Creating repositories for components");
    let task_repo = create_repositories(&db_pools);

    // Initialize optional components based on configuration
    let mut components = AppComponents {
        server: None,
        update_consumer: None,
        task_cleanup_job: None,
    };

    // Setup task event consumer if enabled
    if config.enable_relay_task_consumer {
        debug!(
            broker_url = %config.broker_url,
            queue = %RELAY_QUEUE,
            "Setting up message broker consumer"
        );
        let update_consumer = match RabbitMQTaskEventConsumer::new(
            &config.broker_url,
            Arc::new(task_repo.clone()),
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
        components.update_consumer = Some(Arc::new(update_consumer));
    } else {
        info!("Task event consumer is disabled by configuration");
    }

    // Setup cleanup job if enabled
    if config.enable_relay_cleanup {
        debug!("Creating task cleanup job with 5-minute interval");
        components.task_cleanup_job = Some(Arc::new(TaskCleanupJob::new(
            task_repo.clone(),
            300, // Every 5 minutes
        )));
        info!("Task cleanup job created with 300-second interval");
    } else {
        info!("Task cleanup job is disabled by configuration");
    }

    // Setup API server if enabled
    if config.enable_relay_api {
        let broker_core = match components.update_consumer.as_ref() {
            Some(consumer) => Some(Arc::new(consumer.channel().await?)),
            None => None,
        };

        // Setup axum app and state
        debug!("Setting up web application");
        let app = setup_app(&db_pools, broker_core).await;

        // Create server
        debug!("Creating HTTP server on port 3000");
        let shutdown_rx = shutdown_signal.subscribe();
        components.server = Some(Server::new(app, 3000, shutdown_rx));
        info!(port = 3000, "HTTP server created");
    } else {
        info!("API server is disabled by configuration");
    }

    info!("System components initialized according to configuration");
    Ok(components)
}

/// Starts all enabled application background tasks
///
/// Returns handles and a channel to coordinate shutdown
///
/// # Arguments
///
/// * `components` - The application components
/// * `shutdown_rx` - Oneshot receiver for shutdown coordination
pub async fn start_background_tasks(
    components: AppComponents,
) -> (
    Vec<tokio::task::JoinHandle<()>>,
    Option<Arc<RabbitMQTaskEventConsumer>>,
) {
    debug!("Starting enabled background tasks");
    let mut handles = Vec::new();
    let mut update_consumer_shutdown = None;

    // Start task cleanup job if enabled
    if let Some(cleanup_job) = components.task_cleanup_job {
        info!("Starting task cleanup job");
        let cleanup_handle = tokio::spawn(async move {
            debug!("Task cleanup job started");
            if let Err(e) = cleanup_job.run().await {
                error!(error = %e, "Task cleanup job failed");
            } else {
                info!("Task cleanup job completed successfully");
            }
        });
        handles.push(cleanup_handle);
    }

    // Start update consumer if enabled
    if let Some(consumer) = components.update_consumer {
        // Keep a reference for shutdown
        update_consumer_shutdown = Some(consumer.clone());

        info!("Starting update consumer");
        let consumer_handle = tokio::spawn(async move {
            debug!("Update consumer started");
            if let Err(e) = consumer.lifecycle().await {
                error!(error = %e, "Update consumer failed");
            } else {
                info!("Update consumer completed successfully");
            }
        });
        handles.push(consumer_handle);
    }

    // Start server if enabled
    if let Some(server) = components.server {
        info!("Starting HTTP server");
        let server_handle = tokio::spawn(async move {
            debug!("HTTP server started");
            if let Err(e) = server.run().await {
                error!(error = %e, "Server error");
            } else {
                info!("HTTP server shut down gracefully");
            }
        });
        handles.push(server_handle);
    }

    if handles.is_empty() {
        warn!("No service components were enabled to start");
    } else {
        info!("All enabled background tasks started");
    }

    (handles, update_consumer_shutdown)
}

/// Performs graceful shutdown of all components
///
/// # Arguments
///
/// * `update_consumer` - The update consumer to shut down
pub fn perform_shutdown(update_consumer: Arc<RabbitMQTaskEventConsumer>) {
    info!("Starting graceful shutdown procedure");

    debug!("Shutting down update consumer");
    match update_consumer.shutdown() {
        Ok(_) => info!("Update consumer shut down successfully"),
        Err(e) => error!(error = %e, "Failed to shutdown update consumer"),
    }

    info!("All components shut down, cleanup complete");
}
