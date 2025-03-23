use crate::brokers::core::BrokerProducer;
use crate::brokers::setup_consumer_broker;
use crate::constants::{RELAY_EXCHANGE, RELAY_QUEUE};
use crate::controller::task;
use crate::jobs::TaskCleanupJob;
use crate::models::Task;
use crate::repo::{PgRepositoryCore, PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository};
use crate::server::Server;
use crate::{AppState, Config};
use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use backoff::ExponentialBackoffBuilder;
use sqlx::PgPool;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Represents components that may be initialized depending on configuration
pub struct AppComponents {
    pub server: Option<Server>,
    pub task_controller: Option<Arc<task::TaskController>>,
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
pub fn create_repositories(
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
pub async fn setup_app_state(
    db_pools: &PgPool,
    task_producer: Arc<dyn BrokerProducer<Task>>,
) -> AppState {
    debug!("Setting up application state");
    let (task_repository, worker_kind_repository, worker_repository) =
        create_repositories(db_pools);

    info!("Application state initialized successfully");
    AppState {
        task_repository,
        worker_kind_repository,
        worker_repository,
        task_producer,
    }
}

/// Initializes the application router
///
/// Also injects it with tracing middleware to create spans across the application
///
/// # Arguments
///
/// * `db_pools` - The database connection pools
pub async fn setup_app(db_pools: &PgPool, task_producer: Arc<dyn BrokerProducer<Task>>) -> Router {
    debug!("Beginning app setup");
    let app_state = setup_app_state(db_pools, task_producer).await;
    info!("App state created");

    // Create base router with routes and state
    debug!("Creating router with OpenTelemetry layers");
    let router = Router::new()
        .merge(crate::api::routes())
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
) -> Result<AppComponents, Box<dyn std::error::Error>> {
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
    let (task_repo, worker_kind_repo, worker_repo) = create_repositories(&db_pools);

    // Initialize optional components based on configuration
    let mut components = AppComponents {
        server: None,
        task_controller: None,
        task_cleanup_job: None,
    };

    // Setup task controller if enabled
    if config.enable_relay_task_consumer {
        debug!(
            broker_url = %config.broker_url,
            queue = %RELAY_QUEUE,
            "Setting up message broker consumer"
        );

        // Since retries are now handled in setup_consumer_broker, just call the function directly
        let new_task_consumer =
            match setup_consumer_broker::<Task>(&config.broker_url, RELAY_QUEUE, shutdown.clone())
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

        debug!("Creating task controller");
        match task::TaskController::new(
            new_task_consumer,
            worker_repo,
            worker_kind_repo,
            task_repo.clone(),
        )
        .await
        {
            Ok(controller) => {
                info!("Task controller initialized successfully");
                components.task_controller = Some(Arc::new(controller));
            }
            Err(e) => {
                error!(error = %e, "Failed to initialize task controller");
                return Err(e);
            }
        }
    } else {
        info!("Task consumer is disabled by configuration");
    }

    // Setup cleanup job if enabled
    if config.enable_relay_cleanup {
        debug!("Creating task cleanup job with 5-minute interval");
        components.task_cleanup_job = Some(Arc::new(TaskCleanupJob::new(
            PgTaskRepository::new(PgRepositoryCore::new(db_pools.clone())),
            300, // Every 5 minutes
        )));
        info!("Task cleanup job created with 300-second interval");
    } else {
        info!("Task cleanup job is disabled by configuration");
    }

    // Setup API server if enabled
    if config.enable_relay_api {
        // Create the task publisher for the API server -> currently only used for health checks
        debug!("Setting up task publisher for API server");
        let task_publisher = match crate::brokers::setup_publisher_broker::<Task>(
            &config.broker_url,
            RELAY_EXCHANGE,
        )
        .await
        {
            Ok(publisher) => {
                info!("Task publisher initialized successfully");
                publisher
            }
            Err(e) => {
                error!(error = %e, "Failed to initialize task publisher");
                return Err(e);
            }
        };

        // Setup axum app and state
        debug!("Setting up web application");
        let app = setup_app(&db_pools, task_publisher).await;

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
/// * `shutdown_signal` - Broadcast channel for shutdown coordination
pub async fn start_background_tasks(
    components: AppComponents,
    shutdown_signal: broadcast::Sender<()>,
) -> (
    Vec<tokio::task::JoinHandle<()>>,
    Option<Arc<task::TaskController>>,
) {
    debug!("Starting enabled background tasks");
    let mut handles = Vec::new();
    let mut task_controller_shutdown = None;

    // Start task cleanup job if enabled
    if let Some(cleanup_job) = components.task_cleanup_job {
        info!("Starting task cleanup job");
        let mut shutdown_rx = shutdown_signal.subscribe();
        let task_cleanup_handle = tokio::spawn(async move {
            debug!("Task cleanup job started");
            tokio::select! {
                result = cleanup_job.run() => {
                    if let Err(e) = result {
                        error!(error = %e, "Task cleanup job failed");
                    } else {
                        info!("Task cleanup job completed successfully");
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Task cleanup job received shutdown signal");
                }
            }
        });
        handles.push(task_cleanup_handle);
    }

    // Start task controller if enabled
    if let Some(controller) = components.task_controller {
        // Keep a reference for shutdown
        task_controller_shutdown = Some(controller.clone());

        info!("Starting task controller");
        let mut shutdown_rx = shutdown_signal.subscribe();
        let task_handle = tokio::spawn(async move {
            debug!("Task controller started");
            tokio::select! {
                result = controller.clone().run() => {
                    if let Err(e) = result {
                        error!(error = %e, "Task input controller failed");
                    } else {
                        info!("Task controller completed successfully");
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Task controller received shutdown signal");
                    if let Err(e) = controller.shutdown().await {
                        error!(error = %e, "Failed to shutdown task controller gracefully");
                    }
                }
            }
        });
        handles.push(task_handle);
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

    (handles, task_controller_shutdown)
}

/// Performs graceful shutdown of all components
///
/// # Arguments
///
/// * `task_controller` - The task controller to shut down (if running)
pub async fn perform_shutdown(task_controller: Option<Arc<task::TaskController>>) {
    info!("Starting graceful shutdown procedure");

    if let Some(controller) = task_controller {
        debug!("Shutting down task controller");
        match controller.shutdown().await {
            Ok(_) => info!("Task controller shut down successfully"),
            Err(e) => error!(error = %e, "Failed to shutdown task input controller"),
        }
    }

    info!("All components shut down, cleanup complete");
}
