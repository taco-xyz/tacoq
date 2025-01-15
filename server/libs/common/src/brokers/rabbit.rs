use crate::brokers::core::{BrokerCore, MessageHandler};
use async_trait::async_trait;
use futures::StreamExt;
use lapin::{
    options::*,
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RabbitBroker {
    channel: Channel,
}

impl RabbitBroker {
    pub async fn new(url_string: &str) -> Result<RabbitBroker, Box<dyn std::error::Error>> {
        let connection = Connection::connect(url_string, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        Ok(Self { channel })
    }
}

#[async_trait]
impl BrokerCore for RabbitBroker {
    async fn register_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.channel
            .exchange_declare(
                exchange,
                ExchangeKind::Direct,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }

    async fn register_queue(
        &self,
        exchange: Option<String>,
        queue: &str,
        routing_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;

        if let Some(ex) = exchange {
            self.channel
                .queue_bind(
                    queue,
                    &ex,
                    routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        Ok(())
    }

    async fn delete_queue(&self, queue: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.channel
            .queue_delete(queue, QueueDeleteOptions::default())
            .await?;

        Ok(())
    }

    async fn delete_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.channel
            .exchange_delete(exchange, ExchangeDeleteOptions::default())
            .await?;

        Ok(())
    }

    async fn publish_message(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
        message_id: &str,
        task_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize headers
        let mut headers = FieldTable::default();
        headers.insert("task_kind".into(), AMQPValue::LongString(task_id.into()));

        self.channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default()
                    .with_message_id(message_id.into())
                    .with_headers(headers),
            )
            .await?;

        Ok(())
    }

    async fn consume_messages(
        &self,
        queue: &str,
        handler: Box<dyn MessageHandler>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut consumer = self
            .channel
            .basic_consume(
                queue,
                "manager",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if shutdown.load(Ordering::SeqCst) {
                break;
            }

            let message = delivery.expect(&format!("Error in consumer {}", queue));
            let payload = message.data;
            handler.handle(payload)?;
            self.channel
                .basic_ack(message.delivery_tag, BasicAckOptions::default())
                .await?;
        }

        Ok(())
    }
}
