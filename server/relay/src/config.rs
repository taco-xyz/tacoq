use dotenv::dotenv;
use tracing::{debug, error, info, warn};

pub struct Config {
    pub broker_addr: String,
    pub db_reader_url: String,
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

        let broker_addr = match std::env::var("TACOQ_BROKER_ADDR") {
            Ok(val) => {
                debug!(broker_addr = %val, "Loaded broker address");
                val
            }
            Err(e) => {
                error!(error = %e, "Failed to load TACOQ_BROKER_ADDR environment variable");
                panic!("Environment variable TACOQ_BROKER_ADDR is missing");
            }
        };

        let db_reader_url = match std::env::var("TACOQ_DATABASE_READER_URL") {
            Ok(val) => {
                debug!(db_url_length = val.len(), "Loaded database reader URL");
                val
            }
            Err(e) => {
                error!(error = %e, "Failed to load TACOQ_DATABASE_READER_URL environment variable");
                panic!("Environment variable TACOQ_DATABASE_READER_URL is missing");
            }
        };

        info!("Application configuration initialized successfully");
        Config {
            broker_addr,
            db_reader_url,
        }
    }
}
