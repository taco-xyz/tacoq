use dotenv::dotenv;
use tracing::{debug, error, info, warn};

pub struct Config {
    pub broker_url: String,
    pub db_url: String,
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

        let broker_url = match std::env::var("BROKER_URL") {
            Ok(val) => {
                debug!(broker_url = %val, "Loaded broker address");
                val
            }
            Err(e) => {
                error!(error = %e, "Failed to load BROKER_URL environment variable");
                panic!("Environment variable BROKER_URL is missing");
            }
        };

        let db_url = match std::env::var("DATABASE_URL") {
            Ok(val) => {
                debug!(db_url_length = val.len(), "Loaded database reader URL");
                val
            }
            Err(e) => {
                error!(error = %e, "Failed to load DATABASE_URL environment variable");
                panic!("Environment variable DATABASE_URL is missing");
            }
        };

        info!("Application configuration initialized successfully");
        Config { broker_url, db_url }
    }
}
