use crate::constants;
use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::BrokerConsumer;
use common::brokers::rabbit::{RabbitBrokerCore, TaskResultRabbitMQConsumer};
use common::TaskResult;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskResultController {
    consumer: TaskResultRabbitMQConsumer,
    _task_repository: Arc<PgTaskInstanceRepository>,
}

impl TaskResultController {
    pub async fn new(
        broker_url: &str,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let core = RabbitBrokerCore::new(broker_url).await?;
        let consumer = TaskResultRabbitMQConsumer::new(
            core,
            constants::TASK_RESULT_QUEUE,
            Arc::new(AtomicBool::new(false)),
        )
        .await?;

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
