use async_trait::async_trait;
use mockall::automock;

pub trait MessageHandler: Send {
    fn handle(&self, message: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;
}

#[automock]
#[async_trait]
pub trait BrokerCore: Send + Sync {
    async fn register_exchange(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>>;

    async fn register_queue(
        &self,
        exchange: &str,
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
    ) -> Result<(), Box<dyn std::error::Error>>;
}
