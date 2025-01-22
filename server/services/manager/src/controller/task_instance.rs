use crate::repo::impls::task_instance_repo::PgTaskInstanceRepository;
use common::brokers::core::BrokerConsumer;
use common::models::Task;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NewTaskController {
    consumer: Arc<dyn BrokerConsumer<Task>>,
    _task_repository: Arc<PgTaskInstanceRepository>,
}

impl NewTaskController {
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
        // let producer = self.producer.clone();

        let handler = Box::new(move |task: Task| {
            // Here we would process the input and create a new task instance
            println!("Received task input: {:?}", task);
            // Example of publishing (you'll want to implement actual logic)
            // producer.publish_message(new_task).await?;
            Ok(())
        });

        self.consumer.consume_messages(handler).await
    }
}
