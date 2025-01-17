use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub type MessageHandlerFn<T> =
    Box<dyn Fn(T) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>;

#[automock]
#[async_trait]
pub trait BrokerConsumer<T: Send + Sync + 'static> {
    async fn consume_messages(
        &self,
        handler: MessageHandlerFn<T>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[automock]
#[async_trait]
pub trait BrokerProducer<T: Send + Sync>: Send + Sync + Debug {
    async fn publish_message(&self, message: &T) -> Result<(), Box<dyn std::error::Error>>;
}
