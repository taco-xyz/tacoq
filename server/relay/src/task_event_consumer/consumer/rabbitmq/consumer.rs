use crate::repo::task_repo::TaskRepository;
use crate::task_event_consumer::{
    event_parsing::Event, handler::TaskEventHandler, TaskEventConsumer,
};
use futures::StreamExt;
use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use lapin::{options::QueueDeclareOptions, Connection, ConnectionProperties};
use lapin::{Channel, Consumer};
use std::error::Error;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use tracing::{debug, error, info, warn};

static QUEUE_NAME: &str = "relay_queue";

/// A consumer that listens for task events from RabbitMQ and uploads
/// them to the task repository.
pub struct RabbitMQTaskEventConsumer {
    event_handler: TaskEventHandler,
    url_string: String,
    shutdown: Arc<AtomicBool>,
}

impl RabbitMQTaskEventConsumer {
    /// Creates a new RabbitMQ task event consumer. Does not connect directly
    /// as that is instead done in the `lifecycle` method.
    pub fn new(
        url_string: &str,
        task_repository: Arc<TaskRepository>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            url_string: url_string.to_string(),
            event_handler: TaskEventHandler::new(task_repository),
            shutdown,
        })
    }

    /// Creates a new RabbitMQ channel.
    async fn channel(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let connection =
            match Connection::connect(self.url_string.as_str(), ConnectionProperties::default())
                .await
            {
                Ok(conn) => {
                    debug!("RabbitMQ connection established successfully");
                    conn
                }
                Err(e) => {
                    error!(error = %e, url = %self.url_string, "Failed to connect to RabbitMQ");
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

        Ok(channel)
    }

    /// Creates a new RabbitMQ consumer based on a channel.
    async fn consumer(&self, channel: &Channel) -> Result<Consumer, Box<dyn Error + Send + Sync>> {
        info!(url = %self.url_string, queue = %QUEUE_NAME, "Connecting to RabbitMQ for consumer");

        let mut arguments = FieldTable::default();
        arguments.insert("x-max-priority".into(), 255.into());

        debug!(queue = %QUEUE_NAME, "Declaring queue with priority support");
        match channel
            .queue_declare(
                QUEUE_NAME,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                arguments,
            )
            .await
        {
            Ok(_) => debug!(queue = %QUEUE_NAME, "Queue declared successfully"),
            Err(e) => {
                error!(error = %e, queue = %QUEUE_NAME, "Failed to declare queue");
                return Err(Box::new(e));
            }
        };

        info!(queue = %QUEUE_NAME, "RabbitMQ consumer setup complete");

        let consumer = match channel
            .basic_consume(
                QUEUE_NAME,
                "relay",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(consumer) => {
                info!(queue = %QUEUE_NAME, "Consumer registered successfully, waiting for messages");
                consumer
            }
            Err(e) => {
                error!(error = %e, queue = %QUEUE_NAME, "Failed to register consumer");
                return Err(Box::new(e));
            }
        };

        Ok(consumer)
    }
}

impl TaskEventConsumer for RabbitMQTaskEventConsumer {
    fn event_handler(&self) -> &TaskEventHandler {
        &self.event_handler
    }

    async fn lifecycle(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(queue = %QUEUE_NAME, "Starting message consumption");

        let channel = match self.channel().await {
            Ok(ch) => ch,
            Err(e) => {
                error!(error = %e, queue = %QUEUE_NAME, "Failed to create channel");
                return Err(e);
            }
        };

        let mut consumer = match self.consumer(&channel).await {
            Ok(consumer) => consumer,
            Err(e) => {
                error!(error = %e, queue = %QUEUE_NAME, "Failed to create consumer");
                return Err(e);
            }
        };

        while let Some(delivery) = consumer.next().await {
            // Check for shutdown signal every time a message is received
            if self.shutdown.load(Ordering::SeqCst) {
                warn!("Shutting down task event consumer due to shutdown signal");
                break;
            }

            // Receive message
            let message: Delivery = match delivery {
                Ok(msg) => msg,
                Err(e) => {
                    error!(error = %e, "Error receiving message");
                    continue;
                }
            };
            let delivery_tag = message.delivery_tag;

            // Parse the Event from the message
            let event = match Event::try_from(message) {
                Ok(msg) => msg,
                Err(e) => {
                    error!(error = %e, "Error parsing message");
                    continue;
                }
            };

            // Handle the event. If it fails, we log it, nack it, and continue.
            if let Err(e) = self.handle_events(vec![event]).await {
                error!(error = %e, "Error handling events");
                continue;
            }

            // Ackowledge the message so we don't re-process it.
            debug!(queue = %QUEUE_NAME, delivery_tag = %delivery_tag, "Acknowledging message");
            if let Err(e) = channel
                .basic_ack(delivery_tag, BasicAckOptions::default())
                .await
            {
                error!(
                    error = %e,
                    delivery_tag = %delivery_tag,
                    "Failed to acknowledge message"
                );
            }
        }

        Ok(())
    }

    fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(queue = %QUEUE_NAME, "Initiating consumer shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        debug!(queue = %QUEUE_NAME, "Shutdown flag set");
        Ok(())
    }
}
