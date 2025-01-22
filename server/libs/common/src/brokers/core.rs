use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;
use std::marker::{Send, Sync};

// The message handler function serves as a callback for consumed messages
// It is expected to return a result indicating if the message was processed successfully
pub type MessageHandlerFn<T> =
    Box<dyn Fn(T) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>;

#[automock]
#[async_trait]
pub trait BrokerConsumer<T: Send + Sync + 'static>: Send + Sync + Debug {
    async fn consume_messages(
        &self,
        handler: MessageHandlerFn<T>, // The callback is used when consuming a message
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[automock]
#[async_trait]
pub trait BrokerProducer<T: Send + Sync>: Send + Sync + Debug {
    async fn publish_message(&self, message: &T) -> Result<(), Box<dyn std::error::Error>>;
}
