use backoff::ExponentialBackoffBuilder;
use lapin::{Connection, ConnectionProperties};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

#[derive(Clone)]
pub struct RabbitMQConnection {
    connection: Arc<Connection>,
    url: String,
}

impl RabbitMQConnection {
    pub async fn new(url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_secs(1))
            .with_max_interval(Duration::from_secs(10))
            .build();

        let connection = match backoff::future::retry(backoff, || async {
            match Connection::connect(url, ConnectionProperties::default()).await {
                Ok(conn) => Ok(conn),
                Err(e) => {
                    debug!(error = %e, "Failed to connect to RabbitMQ, retrying...");
                    Err(backoff::Error::transient(e))
                }
            }
        })
        .await
        {
            Ok(conn) => {
                debug!("RabbitMQ connection established successfully");
                conn
            }
            Err(e) => {
                error!(error = %e, url = %url, "Failed to connect to RabbitMQ after retries");
                return Err(Box::new(e));
            }
        };

        Ok(Self {
            connection: Arc::new(connection),
            url: url.to_string(),
        })
    }

    pub async fn reconnect(&self) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_secs(1))
            .with_max_interval(Duration::from_secs(10))
            .build();

        match backoff::future::retry(backoff, || async {
            match Connection::connect(&self.url, ConnectionProperties::default()).await {
                Ok(conn) => Ok(conn),
                Err(e) => {
                    debug!(error = %e, "Failed to reconnect to RabbitMQ, retrying...");
                    Err(backoff::Error::transient(e))
                }
            }
        })
        .await
        {
            Ok(conn) => {
                debug!("RabbitMQ connection re-established successfully");
                Ok(Self {
                    connection: Arc::new(conn),
                    url: self.url.clone(),
                })
            }
            Err(e) => {
                error!(error = %e, url = %self.url, "Failed to reconnect to RabbitMQ after retries");
                Err(Box::new(e))
            }
        }
    }

    pub async fn create_channel(&self) -> Result<lapin::Channel, Box<dyn Error + Send + Sync>> {
        match self.connection.create_channel().await {
            Ok(ch) => {
                debug!("Created new RabbitMQ channel");
                Ok(ch)
            }
            Err(e) => {
                error!(error = %e, "Failed to create RabbitMQ channel");
                Err(Box::new(e))
            }
        }
    }
}
