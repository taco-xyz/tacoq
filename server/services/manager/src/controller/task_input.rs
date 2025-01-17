use crate::constants;
use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::BrokerConsumer;
use common::brokers::rabbit::{
    RabbitBrokerCore, TaskInstanceRabbitMQProducer, TaskResultRabbitMQConsumer,
};
use common::TaskResult;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskInputController {
    consumer: TaskResultRabbitMQConsumer,
    producer: TaskInstanceRabbitMQProducer,
    _task_repository: Arc<PgTaskInstanceRepository>,
}

impl TaskInputController {
    pub async fn new(
        broker_url: &str,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let core = RabbitBrokerCore::new(broker_url).await?;

        let consumer = TaskResultRabbitMQConsumer::new(
            core.clone(),
            constants::TASK_INPUT_QUEUE,
            Arc::new(AtomicBool::new(false)),
        )
        .await?;

        let producer =
            TaskInstanceRabbitMQProducer::new(core, constants::TASK_OUTPUT_EXCHANGE).await?;

        Ok(Self {
            consumer,
            producer,
            _task_repository: task_repository,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // let producer = self.producer.clone();

        let handler = Box::new(move |task: TaskResult| {
            // Here we would process the input and create a new task instance
            println!("Received task input: {:?}", task);
            // Example of publishing (you'll want to implement actual logic)
            // producer.publish_message(new_task).await?;
            Ok(())
        });

        self.consumer.consume_messages(handler).await
    }

    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.cleanup().await?;
        self.producer.cleanup().await?;
        Ok(())
    }
}
