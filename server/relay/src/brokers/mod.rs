pub mod core;
pub mod rabbit;

use backoff::ExponentialBackoffBuilder;
use core::{BrokerConsumer, BrokerProducer};
use rabbit::{setup_rabbit_consumer, setup_rabbit_producer};
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tracing::warn;

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
    // Configure backoff strategy
    let backoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(10))
        .build();

    // Try to connect with retries
    backoff::future::retry(backoff, || async {
        match url_str.split_once("://") {
            Some(("amqp", _)) => match setup_rabbit_producer::<T>(url_str, exchange).await {
                Ok(producer) => Ok(producer as Arc<dyn BrokerProducer<T>>),
                Err(e) => {
                    warn!(error = %e, "Failed to connect to message broker producer, retrying...");
                    Err(backoff::Error::transient(e))
                }
            },
            _ => Err(backoff::Error::permanent("Unsupported broker".into())),
        }
    })
    .await
    .map_err(|e| e.into())
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
    // Configure backoff strategy
    let backoff = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(10))
        .build();

    // Try to connect with retries
    backoff::future::retry(backoff, || async {
        match url_str.split_once("://") {
            Some(("amqp", _)) => {
                match setup_rabbit_consumer::<T>(url_str, queue, shutdown.clone()).await {
                    Ok(consumer) => Ok(consumer as Arc<dyn BrokerConsumer<T>>),
                    Err(e) => {
                        warn!(error = %e, "Failed to connect to message broker consumer, retrying...");
                        Err(backoff::Error::transient(e))
                    }
                }
            },
            _ => Err(backoff::Error::permanent("Unsupported broker".into())),
        }
    }).await.map_err(|e| e.into())
}
