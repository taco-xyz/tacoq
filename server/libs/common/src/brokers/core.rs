use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub trait MessageHandler: Send + Sync + 'static {
    fn handle(&self, message: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;
}

#[automock]
#[async_trait]
pub trait BrokerCore: Send + Sync + Debug + 'static {
    async fn register_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn register_queue(
        &self,
        exchange: Option<String>,
        queue: &str,
        routing_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn delete_queue(&self, queue: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn delete_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn publish_message(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &[u8],
        message_id: &str,
        task_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn consume_messages(
        &self,
        queue: &str,
        handler: Box<dyn MessageHandler>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
