use lapin::{Connection, ConnectionProperties};
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Clone)]
pub struct RabbitMQConnection {
    connection: Arc<Connection>,
}

impl RabbitMQConnection {
    pub async fn new(url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connection = match Connection::connect(url, ConnectionProperties::default()).await {
            Ok(conn) => {
                debug!("RabbitMQ connection established successfully");
                conn
            }
            Err(e) => {
                error!(error = %e, url = %url, "Failed to connect to RabbitMQ");
                return Err(Box::new(e));
            }
        };

        Ok(Self {
            connection: Arc::new(connection),
        })
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
