use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::BrokerConsumer;
use common::models::Task;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskResultController {
    consumer: Arc<dyn BrokerConsumer<Task>>,
    _task_repository: Arc<PgTaskInstanceRepository>,
}

impl TaskResultController {
    pub async fn new(
        consumer: Arc<dyn BrokerConsumer<Task>>,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            consumer,
            _task_repository: task_repository,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handler = Box::new(|result: Task| {
            println!("Received task result: {:?}", result);
            Ok(())
        });

        self.consumer.consume_messages(handler).await
    }
}
