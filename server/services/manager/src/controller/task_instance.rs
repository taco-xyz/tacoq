use crate::repo::impls::task_repo::PgTaskRepository;
use common::brokers::core::BrokerConsumer;
use common::models::Task;
use futures::future::BoxFuture;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NewTaskController {
    consumer: Arc<dyn BrokerConsumer<Task>>,
    _task_repository: Arc<PgTaskRepository>,
}

impl NewTaskController {
    pub async fn new(
        consumer: Arc<dyn BrokerConsumer<Task>>,
        task_repository: Arc<PgTaskRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            consumer,
            _task_repository: task_repository,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handler = Box::new(
            |result: Task| -> BoxFuture<'_, Result<(), Box<dyn std::error::Error>>> {
                Box::pin(async move {
                    println!("Received task input: {:?}", result);
                    Ok(())
                })
            },
        );

        self.consumer.consume_messages(handler).await
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.shutdown().await
    }
}
