use crate::consumer::message_types::{Message, MessageType};
use crate::models::{
    AvroSerializable, TaskAssignmentUpdate, TaskCompletedUpdate, TaskRunningUpdate,
};
use crate::repo::TaskRepository;
use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};
use std::error::Error;
use std::{
    clone::Clone,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tracing::{debug, error, info, warn};

/// Errors that can occur when processing a message.
#[derive(Debug, thiserror::Error)]
pub enum MessageConsumptionError {
    #[error("No headers found in message")]
    NoHeadersFound,
    #[error("No message_type found in headers")]
    NoMessageTypeFound,
}

/// A consumer of a RabbitMQ queue, continuously consuming messages, interpreting
/// their type based on the headers, and then handling them.
///
/// # Fields
/// * `channel` - The channel to consume messages from
/// * `queue` - The queue to consume messages from
/// * `shutdown` - A flag indicating if the application is running
#[derive(Clone, Debug)]
pub struct Consumer {
    channel: Channel,
    task_repository: Arc<TaskRepository>,
    queue: String,
    shutdown: Arc<AtomicBool>,
}

impl Consumer {
    /// Creates a new consumer
    ///
    /// # Arguments
    /// * `url_string` - The URL of the RabbitMQ server
    /// * `queue` - The queue to consume messages from
    /// * `shutdown` - A flag indicating if the application is running
    pub async fn new(
        url_string: &str,
        queue: &str,
        task_repository: Arc<TaskRepository>,
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
            task_repository,
        })
    }

    /// Consumes messages from the queue and handles them based on their
    /// `message_type` which is specified in the headers.
    ///
    /// # Returns
    /// A result indicating if the consumption was successful
    pub async fn consume_messages(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(queue = %self.queue, "Starting message consumption");
        let mut consumer = match self
            .channel
            .basic_consume(
                &self.queue,
                "relay",
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
            // Check for shutdown signal
            if self.shutdown.load(Ordering::SeqCst) {
                warn!(queue = %self.queue, "Shutting down consumer due to shutdown signal");
                break;
            }

            // Receive message
            let message = match delivery {
                Ok(msg) => msg,
                Err(e) => {
                    error!(error = %e, queue = %self.queue, "Error receiving message");
                    continue;
                }
            };

            // Get payload and headers
            let payload = message.data;
            let delivery_tag = message.delivery_tag;

            debug!(
                queue = %self.queue,
                delivery_tag = %delivery_tag,
                headers = ?message.properties.headers(),
                payload_size = payload.len(),
                "Received message"
            );

            // Get the message type from the headers
            let message_type: Result<MessageType, Box<dyn Error + Send + Sync>> =
                match message.properties.headers() {
                    Some(headers) => match headers.inner().get("message_type") {
                        Some(message_type) => message_type
                            .as_long_string()
                            .ok_or_else(|| MessageConsumptionError::NoMessageTypeFound)
                            .map(|s| s.to_string())
                            .and_then(|s| {
                                s.try_into()
                                    .map_err(|_| MessageConsumptionError::NoMessageTypeFound)
                            })
                            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>),
                        None => Err(MessageConsumptionError::NoMessageTypeFound.into()),
                    },
                    None => Err(MessageConsumptionError::NoHeadersFound.into()),
                };

            // Handle errors and deserialize the payload based on the message type
            let result: Result<Message, Box<dyn Error + Send + Sync>> = match message_type {
                Ok(message_type) => match message_type {
                    MessageType::TaskCompleted => {
                        TaskCompletedUpdate::try_from_avro_bytes(&payload)
                            .map(Message::TaskCompleted)
                    }
                    MessageType::TaskAssignment => {
                        TaskAssignmentUpdate::try_from_avro_bytes(&payload)
                            .map(Message::TaskAssignment)
                    }
                    MessageType::TaskRunning => {
                        TaskRunningUpdate::try_from_avro_bytes(&payload).map(Message::TaskRunning)
                    }
                },
                Err(e) => Err(e),
            };

            // Handle the update
            match result {
                Ok(update) => match update {
                    Message::TaskAssignment(update) => {
                        info!(queue = %self.queue, delivery_tag = %delivery_tag, "Task assignment update received");
                        self.task_repository
                            .update_task_from_assignment_update(&update)
                            .await
                            .unwrap();
                    }
                    Message::TaskCompleted(update) => {
                        info!(queue = %self.queue, delivery_tag = %delivery_tag, "Task completed update received");
                        self.task_repository
                            .update_task_from_completed_update(&update)
                            .await
                            .unwrap();
                    }
                    Message::TaskRunning(update) => {
                        info!(queue = %self.queue, delivery_tag = %delivery_tag, "Task running update received");
                        self.task_repository
                            .update_task_from_running_update(&update)
                            .await
                            .unwrap();
                    }
                },
                Err(e) => {
                    error!(
                        error = %e,
                        queue = %self.queue,
                        delivery_tag = %delivery_tag,
                        "Failed to handle message"
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

    /// Shuts down the consumer
    ///
    /// # Returns
    /// A result indicating if the shutdown was successful
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(queue = %self.queue, "Initiating consumer shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        debug!(queue = %self.queue, "Shutdown flag set");
        Ok(())
    }
}
