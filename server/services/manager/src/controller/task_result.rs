use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::BrokerConsumer;
use common::TaskResult;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskResultController {
    consumer: Arc<dyn BrokerConsumer<TaskResult>>,
    _task_repository: Arc<PgTaskInstanceRepository>,
}

impl TaskResultController {
    pub async fn new(
        consumer: Arc<dyn BrokerConsumer<TaskResult>>,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            consumer,
            _task_repository: task_repository,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handler = Box::new(|result: TaskResult| {
            println!("Received task result: {:?}", result);
            Ok(())
        });

        self.consumer.consume_messages(handler).await
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.cleanup().await
    }
}
