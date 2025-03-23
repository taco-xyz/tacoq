use dotenv::dotenv;
use tracing::{debug, error, info, warn};

pub struct Config {
    pub broker_url: String,
    pub db_url: String,
    pub enable_relay_task_consumer: bool,
    pub enable_relay_cleanup: bool,
    pub enable_relay_api: bool,
}

fn load_env() {
    // Load only in development
    if cfg!(debug_assertions) {
        debug!("Development mode detected, loading .env file");
        match dotenv() {
            Ok(_) => debug!("Successfully loaded .env file"),
            Err(e) => warn!("Failed to load .env file: {}", e),
        }
    } else {
        debug!("Production mode detected, using environment variables");
    }
}

impl Config {
    pub fn new() -> Config {
        load_env();
        info!("Initializing application configuration");

        let broker_url = match std::env::var("TACOQ_BROKER_URL") {
            Ok(val) => {
                debug!(broker_url = %val, "Loaded broker address");
                val
            }
            Err(e) => {
                error!(error = %e, "Failed to load TACOQ_BROKER_URL environment variable");
                panic!("Environment variable TACOQ_BROKER_URL is missing");
            }
        };

        let db_url = match std::env::var("TACOQ_DATABASE_URL") {
            Ok(val) => {
                debug!(db_url_length = val.len(), "Loaded database reader URL");
                val
            }
            Err(e) => {
                error!(error = %e, "Failed to load TACOQ_DATABASE_URL environment variable");
                panic!("Environment variable TACOQ_DATABASE_URL is missing");
            }
        };

        // If the env var is there log in debug else do nothing
        let enable_relay_task_consumer = std::env::var("TACOQ_ENABLE_RELAY_TASK_CONSUMER")
            .ok()
            .map(|val| {
                debug!(enable_relay_task_consumer = %val, "Loaded enable relay task consumer");
                val.parse::<bool>()
                    .expect("Invalid value for TACOQ_ENABLE_RELAY_TASK_CONSUMER")
            })
            .unwrap_or(true);

        let enable_relay_cleanup = std::env::var("TACOQ_ENABLE_RELAY_CLEANUP")
            .ok()
            .map(|val| {
                debug!(enable_relay_cleanup = %val, "Loaded enable relay cleanup");
                val.parse::<bool>()
                    .expect("Invalid value for TACOQ_ENABLE_RELAY_CLEANUP")
            })
            .unwrap_or(true);

        let enable_relay_api = std::env::var("TACOQ_ENABLE_RELAY_API")
            .ok()
            .map(|val| {
                debug!(enable_relay_api = %val, "Loaded enable relay API");
                val.parse::<bool>()
                    .expect("Invalid value for TACOQ_ENABLE_RELAY_API")
            })
            .unwrap_or(true);

        info!("Application configuration initialized successfully");

        Config {
            broker_url,
            db_url,
            enable_relay_task_consumer,
            enable_relay_cleanup,
            enable_relay_api,
        }
    }
}
