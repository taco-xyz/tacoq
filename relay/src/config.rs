use std::path::PathBuf;

use dotenv::dotenv;
use tracing::{debug, error, info, warn};

pub struct Config {
    pub broker_url: String,
    pub db_url: String,

    // API configuration fields
    pub api_port: u16,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,

    // Service flags
    pub enable_task_consumer: bool,
    pub enable_task_cleanup: bool,
    pub enable_rest_api: bool,
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

        // --- Load API specific config ---
        let relay_api_port = std::env::var("TACOQ_RELAY_API_PORT")
            .ok()
            .map(|val| {
                val.parse::<u16>()
                    .expect("Invalid value for TACOQ_RELAY_API_PORT")
            })
            .unwrap_or(3000); // Default port if not set
        debug!(relay_api_port, "Loaded API port");

        let relay_cert_path = std::env::var("TACOQ_RELAY_CERT_PATH")
            .ok()
            .map(PathBuf::from);
        let relay_key_path = std::env::var("TACOQ_RELAY_KEY_PATH")
            .ok()
            .map(PathBuf::from);

        if relay_cert_path.is_some() && relay_key_path.is_some() {
            info!(
                cert_path = %relay_cert_path.as_ref().unwrap().display(),
                key_path = %relay_key_path.as_ref().unwrap().display(),
                "TLS enabled for API server"
            );
        } else if relay_cert_path.is_some() || relay_key_path.is_some() {
            warn!("TLS requires both TACOQ_RELAY_CERT_PATH and TACOQ_RELAY_KEY_PATH. Falling back to HTTP.");
        } else {
            info!("TLS not configured for API server, using HTTP");
        }

        // --- Load service flags ---
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
            api_port: relay_api_port,
            cert_path: relay_cert_path,
            key_path: relay_key_path,
            enable_task_consumer: enable_relay_task_consumer,
            enable_task_cleanup: enable_relay_cleanup,
            enable_rest_api: enable_relay_api,
        }
    }
}
