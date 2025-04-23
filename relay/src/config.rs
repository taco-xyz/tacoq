use clap::Parser;
use std::path::PathBuf;

use dotenv::dotenv;
use tracing::{debug, error, info, warn};

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(long, short)]
    port: Option<u16>,

    #[arg(long)]
    cert_path: Option<PathBuf>,

    #[arg(long)]
    key_path: Option<PathBuf>,
}

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
    /// Loads mandatory environment variables.
    /// Panics if any required variable is missing.
    fn load_required_env_vars() -> (String, String) {
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

        (broker_url, db_url)
    }

    /// Loads API-specific configuration (port, cert path, key path).
    /// Prioritizes CLI arguments over environment variables.
    fn load_api_config(cli_args: &CliArgs) -> (u16, Option<PathBuf>, Option<PathBuf>) {
        // Port: CLI > Env > Default (3000)
        let relay_api_port = cli_args
            .port
            .or_else(|| {
                std::env::var("TACOQ_RELAY_API_PORT")
                    .ok()
                    .and_then(|val| val.parse::<u16>().ok())
            })
            .unwrap_or(3000);
        debug!(relay_api_port, "Loaded API port");

        let relay_cert_path = cli_args.cert_path.clone().or_else(|| {
            std::env::var("TACOQ_RELAY_CERT_PATH")
                .ok()
                .map(PathBuf::from)
        });

        let relay_key_path = cli_args.key_path.clone().or_else(|| {
            std::env::var("TACOQ_RELAY_KEY_PATH")
                .ok()
                .map(PathBuf::from)
        });

        if relay_cert_path.is_some() && relay_key_path.is_some() {
            info!(
                cert_path = %relay_cert_path.as_ref().unwrap().display(),
                key_path = %relay_key_path.as_ref().unwrap().display(),
                "TLS enabled for API server"
            );
        } else if relay_cert_path.is_some() || relay_key_path.is_some() {
            warn!("TLS requires both certificate and key paths (check CLI args and TACOQ_RELAY_CERT_PATH/TACOQ_RELAY_KEY_PATH env vars). Falling back to HTTP.");
        } else {
            info!("TLS not configured for API server, using HTTP");
        }

        (relay_api_port, relay_cert_path, relay_key_path)
    }

    /// Loads service enablement flags from environment variables.
    /// Defaults to true if variables are not set or invalid.
    fn load_service_flags() -> (bool, bool, bool) {
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

        (
            enable_relay_task_consumer,
            enable_relay_cleanup,
            enable_relay_api,
        )
    }

    pub fn new() -> Config {
        load_env();
        let cli_args = CliArgs::parse();

        info!("Initializing application configuration");

        let (broker_url, db_url) = Self::load_required_env_vars();
        let (api_port, cert_path, key_path) = Self::load_api_config(&cli_args);
        let (enable_task_consumer, enable_task_cleanup, enable_rest_api) =
            Self::load_service_flags();

        info!("Application configuration initialized successfully");

        Config {
            broker_url,
            db_url,
            api_port,
            cert_path,
            key_path,
            enable_task_consumer,
            enable_task_cleanup,
            enable_rest_api,
        }
    }
}
