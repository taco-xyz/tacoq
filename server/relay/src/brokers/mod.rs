pub mod core;
pub mod rabbit;
pub mod testing;

use core::{BrokerConsumer, BrokerProducer};
use rabbit::{setup_rabbit_consumer, setup_rabbit_producer};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use std::fmt::Debug;

/// Initializes a generic producer broker
///
/// # Arguments
///
/// * `config` - The configuration for the broker   
/// * `exchange` - The exchange to publish messages to
pub async fn setup_publisher_broker<T>(
    url_str: &str,
    exchange: &str,
) -> Result<Arc<dyn BrokerProducer<T>>, Box<dyn std::error::Error>>
where
    T: Debug + Send + Sync + serde::Serialize + 'static,
{
    match url_str.split_once("://") {
        Some(("amqp", _)) => Ok(setup_rabbit_producer::<T>(url_str, exchange).await?),
        _ => Err("Unsupported broker".into()),
    }
}

/// Initializes a generic consumer broker
///
/// # Arguments
///
/// * `config` - The configuration for the broker
/// * `queue` - The queue to consume messages from
/// * `is_running` - A flag indicating if the application is running
pub async fn setup_consumer_broker<T>(
    url_str: &str,
    queue: &str,
    shutdown: Arc<AtomicBool>,
) -> Result<Arc<dyn BrokerConsumer<T>>, Box<dyn std::error::Error>>
where
    T: Debug + Send + Sync + serde::de::DeserializeOwned + 'static,
{
    match url_str.split_once("://") {
        Some(("amqp", _)) => Ok(setup_rabbit_consumer::<T>(url_str, queue, shutdown).await?),
        _ => Err("Unsupported broker".into()),
    }
}
