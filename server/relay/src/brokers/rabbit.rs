use super::core::{BrokerConsumer, BrokerProducer, MessageHandlerFn};
use async_trait::async_trait;
use futures::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
    ExchangeKind,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    clone::Clone,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct RabbitMQConsumer<T>
where
    T: Debug,
{
    channel: Channel,
    queue: String,
    shutdown: Arc<AtomicBool>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> RabbitMQConsumer<T>
where
    T: Debug,
{
    pub async fn new(
        url_string: &str,
        queue: &str,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!(url = %url_string, queue = %queue, "Connecting to RabbitMQ for consumer");
        let connection =
            match Connection::connect(url_string, ConnectionProperties::default()).await {
                Ok(conn) => {
                    debug!("RabbitMQ connection established successfully");
                    conn
                }
                Err(e) => {
                    error!(error = %e, url = %url_string, "Failed to connect to RabbitMQ");
                    return Err(Box::new(e));
                }
            };

        let channel = match connection.create_channel().await {
            Ok(ch) => ch,
            Err(e) => {
                error!(error = %e, "Failed to create RabbitMQ channel");
                return Err(Box::new(e));
            }
        };

        let mut arguments = FieldTable::default();
        arguments.insert("x-max-priority".into(), 255.into());

        debug!(queue = %queue, "Declaring queue with priority support");
        match channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                arguments,
            )
            .await
        {
            Ok(_) => debug!(queue = %queue, "Queue declared successfully"),
            Err(e) => {
                error!(error = %e, queue = %queue, "Failed to declare queue");
                return Err(Box::new(e));
            }
        };

        info!(queue = %queue, "RabbitMQ consumer setup complete");
        Ok(Self {
            channel,
            queue: queue.to_string(),
            shutdown,
            _phantom: std::marker::PhantomData,
        })
    }
}

#[async_trait]
impl<T> BrokerConsumer<T> for RabbitMQConsumer<T>
where
    T: Send + Sync + Debug + DeserializeOwned + 'static,
{
    async fn consume_messages(
        &self,
        handler: MessageHandlerFn<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(queue = %self.queue, "Starting message consumption");
        let mut consumer = match self
            .channel
            .basic_consume(
                &self.queue,
                "manager",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(consumer) => {
                info!(queue = %self.queue, "Consumer registered successfully, waiting for messages");
                consumer
            }
            Err(e) => {
                error!(error = %e, queue = %self.queue, "Failed to register consumer");
                return Err(Box::new(e));
            }
        };

        while let Some(delivery) = consumer.next().await {
            if self.shutdown.load(Ordering::SeqCst) {
                warn!(queue = %self.queue, "Shutting down consumer due to shutdown signal");
                break;
            }

            let message = match delivery {
                Ok(msg) => msg,
                Err(e) => {
                    error!(error = %e, queue = %self.queue, "Error receiving message");
                    continue;
                }
            };

            let payload = message.data;
            let delivery_tag = message.delivery_tag;

            debug!(
                queue = %self.queue,
                delivery_tag = %delivery_tag,
                payload_size = payload.len(),
                "Received message"
            );

            match serde_json::from_slice(&payload) {
                Ok(parsed_message) => {
                    debug!(
                        queue = %self.queue,
                        delivery_tag = %delivery_tag,
                        message = ?parsed_message,
                        "Parsed message successfully, processing"
                    );
                    handler(parsed_message).await;
                }
                Err(e) => {
                    error!(
                        queue = %self.queue,
                        delivery_tag = %delivery_tag,
                        error = %e,
                        payload = %String::from_utf8_lossy(&payload),
                        "Failed to deserialize message"
                    );
                }
            }

            debug!(queue = %self.queue, delivery_tag = %delivery_tag, "Acknowledging message");
            if let Err(e) = self
                .channel
                .basic_ack(delivery_tag, BasicAckOptions::default())
                .await
            {
                error!(
                    error = %e,
                    queue = %self.queue,
                    delivery_tag = %delivery_tag,
                    "Failed to acknowledge message"
                );
            }
        }

        info!(queue = %self.queue, "Consumer loop ended");
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(queue = %self.queue, "Initiating consumer shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        debug!(queue = %self.queue, "Shutdown flag set");
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RabbitMQProducer<T>
where
    T: Debug,
{
    channel: Channel,
    exchange: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> RabbitMQProducer<T>
where
    T: Debug,
{
    pub async fn _new(
        url_string: &str,
        exchange: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!(url = %url_string, exchange = %exchange, "Connecting to RabbitMQ for producer");
        let connection =
            match Connection::connect(url_string, ConnectionProperties::default()).await {
                Ok(conn) => {
                    debug!("RabbitMQ connection established successfully");
                    conn
                }
                Err(e) => {
                    error!(error = %e, url = %url_string, "Failed to connect to RabbitMQ");
                    return Err(Box::new(e));
                }
            };

        let channel = match connection.create_channel().await {
            Ok(ch) => ch,
            Err(e) => {
                error!(error = %e, "Failed to create RabbitMQ channel");
                return Err(Box::new(e));
            }
        };

        debug!(exchange = %exchange, "Declaring exchange");
        match channel
            .exchange_declare(
                exchange,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..ExchangeDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await
        {
            Ok(_) => debug!(exchange = %exchange, "Exchange declared successfully"),
            Err(e) => {
                error!(error = %e, exchange = %exchange, "Failed to declare exchange");
                return Err(Box::new(e));
            }
        };

        info!(exchange = %exchange, "RabbitMQ producer setup complete");
        Ok(Self {
            channel,
            exchange: exchange.to_string(),
            _phantom: std::marker::PhantomData,
        })
    }
}

#[async_trait]
impl<T> BrokerProducer<T> for RabbitMQProducer<T>
where
    T: Send + Sync + Serialize + Debug,
{
    async fn publish_message(&self, message: &T) -> Result<(), Box<dyn std::error::Error>> {
        let payload = match serde_json::to_vec(&message) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, message = ?message, "Failed to serialize message");
                return Err(Box::new(e));
            }
        };

        debug!(
            exchange = %self.exchange,
            payload_size = payload.len(),
            message = ?message,
            "Publishing message to RabbitMQ"
        );

        match self
            .channel
            .basic_publish(
                &self.exchange,
                "", //TODO: add appropriate routing key
                BasicPublishOptions::default(),
                payload.as_slice(),
                BasicProperties::default(),
            )
            .await
        {
            Ok(_) => debug!(exchange = %self.exchange, "Message published successfully"),
            Err(e) => {
                error!(
                    error = %e,
                    exchange = %self.exchange,
                    "Failed to publish message"
                );
                return Err(Box::new(e));
            }
        }

        Ok(())
    }
}

pub async fn _setup_rabbit_producer<T>(
    url_string: &str,
    exchange: &str,
) -> Result<Arc<RabbitMQProducer<T>>, Box<dyn std::error::Error>>
where
    T: Debug,
{
    info!(url = %url_string, exchange = %exchange, "Setting up RabbitMQ producer");
    let producer = match RabbitMQProducer::<T>::_new(url_string, exchange).await {
        Ok(p) => p,
        Err(e) => {
            error!(error = %e, url = %url_string, exchange = %exchange, "Failed to set up RabbitMQ producer");
            return Err(e);
        }
    };
    Ok(Arc::new(producer))
}

pub async fn setup_rabbit_consumer<T>(
    url_string: &str,
    queue: &str,
    shutdown: Arc<AtomicBool>,
) -> Result<Arc<RabbitMQConsumer<T>>, Box<dyn std::error::Error>>
where
    T: Debug,
{
    info!(url = %url_string, queue = %queue, "Setting up RabbitMQ consumer");
    let consumer = match RabbitMQConsumer::<T>::new(url_string, queue, shutdown).await {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, url = %url_string, queue = %queue, "Failed to set up RabbitMQ consumer");
            return Err(e);
        }
    };
    Ok(Arc::new(consumer))
}
