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
        let connection = Connection::connect(url_string, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;

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
        let mut consumer = self
            .channel
            .basic_consume(
                &self.queue,
                "manager",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if self.shutdown.load(Ordering::SeqCst) {
                break;
            }

            let message = delivery.unwrap_or_else(|_| panic!("Error in consumer {}", self.queue));
            let payload = message.data;

            let parsed_message = serde_json::from_slice(&payload)?;
            handler(parsed_message)?;

            self.channel
                .basic_ack(message.delivery_tag, BasicAckOptions::default())
                .await?;
        }

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
    pub async fn new(url_string: &str, exchange: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::connect(url_string, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        channel
            .exchange_declare(
                exchange,
                ExchangeKind::Direct,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

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
        let payload = serde_json::to_vec(&message)?;

        self.channel
            .basic_publish(
                &self.exchange,
                "", //TODO: add appropriate routing key
                BasicPublishOptions::default(),
                payload.as_slice(),
                BasicProperties::default(),
            )
            .await?;

        Ok(())
    }
}

pub async fn setup_rabbit_producer<T>(
    url_string: &str,
    exchange: &str,
) -> Result<Arc<RabbitMQProducer<T>>, Box<dyn std::error::Error>>
where
    T: Debug,
{
    Ok(Arc::new(
        RabbitMQProducer::<T>::new(url_string, exchange).await?,
    ))
}

pub async fn setup_rabbit_consumer<T>(
    url_string: &str,
    queue: &str,
    is_running: Arc<AtomicBool>,
) -> Result<Arc<RabbitMQConsumer<T>>, Box<dyn std::error::Error>>
where
    T: Debug,
{
    Ok(Arc::new(
        RabbitMQConsumer::<T>::new(url_string, queue, is_running).await?,
    ))
}
