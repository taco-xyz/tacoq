use crate::brokers::core::BrokerCore;
use async_trait::async_trait;
use lapin::{
    options::*,
    types::{AMQPValue, FieldTable},
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct RabbitBroker {
    connection: Arc<Connection>,
}

impl RabbitBroker {
    pub async fn new(uri: &str) -> Result<RabbitBroker, Box<dyn std::error::Error>> {
        let connection = Connection::connect(uri, ConnectionProperties::default()).await?;

        Ok(RabbitBroker {
            connection: Arc::new(connection),
        })
    }
}

#[async_trait]
impl BrokerCore for RabbitBroker {
    async fn register_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>> {
        let channel = self.connection.create_channel().await?;

        channel
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
        exchange: &str,
        queue: &str,
        routing_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let channel = self.connection.create_channel().await?;

        channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;

        channel
            .queue_bind(
                queue,
                exchange,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }

    async fn delete_queue(&self, queue: &str) -> Result<(), Box<dyn std::error::Error>> {
        let channel = self.connection.create_channel().await?;

        channel
            .queue_delete(queue, QueueDeleteOptions::default())
            .await?;

        Ok(())
    }

    async fn delete_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>> {
        let channel = self.connection.create_channel().await?;

        channel
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
        let channel = self.connection.create_channel().await?;

        // Initialize headers
        let mut headers = FieldTable::default();
        headers.insert("task_kind".into(), AMQPValue::LongString(task_id.into()));

        channel
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
}
