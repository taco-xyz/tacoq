use async_trait::async_trait;

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
}
