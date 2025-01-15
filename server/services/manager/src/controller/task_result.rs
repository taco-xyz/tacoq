use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::MessageHandler;
use common::brokers::Broker;

use std::sync::Arc;

#[derive(Clone, Debug)]
struct Handler {
    task_repository: Arc<PgTaskInstanceRepository>,
}

impl MessageHandler for Handler {
    fn handle(&self, message: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        println!("{:?}", message);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TaskResultController {
    consumer: Broker,
    handler: Handler,
}

impl TaskResultController {
    pub async fn new(
        broker_url: &str,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let s: &str = "task_result";
        let consumer = Broker::new(broker_url, s, None, Some(s.to_string())).await?;
        let handler = Handler { task_repository };

        Ok(Self { consumer, handler })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.setup().await?;
        self.consumer
            .consume(Box::new(self.handler.clone()))
            .await?;
        Ok(())
    }
}
