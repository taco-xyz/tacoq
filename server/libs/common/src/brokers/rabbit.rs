use crate::{TaskInstance, TaskResult};
use futures::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
    ExchangeKind,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use async_trait::async_trait;

use super::core::{BrokerConsumer, BrokerProducer, MessageHandlerFn};

#[derive(Clone, Debug)]
pub struct RabbitBrokerCore {
    channel: Channel,
}

impl RabbitBrokerCore {
    pub async fn new(url_string: &str) -> Result<RabbitBrokerCore, Box<dyn std::error::Error>> {
        let connection = Connection::connect(url_string, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        Ok(Self { channel })
    }

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
}

#[derive(Clone, Debug)]
pub struct TaskResultRabbitMQConsumer {
    core: RabbitBrokerCore,
    queue: String,
    shutdown: Arc<AtomicBool>,
}

impl TaskResultRabbitMQConsumer {
    pub async fn new(
        core: RabbitBrokerCore,
        queue: &str,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        core.register_queue(None, queue, queue).await?;

        Ok(Self {
            core,
            queue: queue.to_string(),
            shutdown,
        })
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.shutdown.store(true, Ordering::SeqCst);
        self.core.delete_queue(&self.queue).await?;

        Ok(())
    }
}

#[async_trait]
impl BrokerConsumer<TaskResult> for TaskResultRabbitMQConsumer {
    async fn consume_messages(
        &self,
        handler: MessageHandlerFn<TaskResult>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut consumer = self
            .core
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

            // This may be unsafe or error out, we have to check if the payload is valid or not
            let task_result = serde_json::from_slice(&payload)?;
            handler(task_result)?;

            self.core
                .channel
                .basic_ack(message.delivery_tag, BasicAckOptions::default())
                .await?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TaskInstanceRabbitMQProducer {
    core: RabbitBrokerCore,
    exchange: String,
}

impl TaskInstanceRabbitMQProducer {
    pub async fn new(
        core: RabbitBrokerCore,
        exchange: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        core.register_exchange(exchange).await?;

        Ok(Self {
            core,
            exchange: exchange.to_string(),
        })
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.core.delete_exchange(&self.exchange).await?;

        Ok(())
    }
}

#[async_trait]
impl BrokerProducer<TaskInstance> for TaskInstanceRabbitMQProducer {
    async fn publish_message(&self, task: &TaskInstance) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_vec(&task.input_data)?;

        // Need to change the routing key to the task kind, future will be worker kind
        // Have to see how to implement that given abstractions
        self.core
            .channel
            .basic_publish(
                &self.exchange,
                "",
                BasicPublishOptions::default(),
                payload.as_slice(),
                BasicProperties::default(),
            )
            .await?;

        Ok(())
    }
}
