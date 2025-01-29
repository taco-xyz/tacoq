use crate::repo::{
    worker_kind_repo, PgTaskRepository, PgWorkerKindRepository, TaskRepository,
    WorkerKindRepository,
};
use common::brokers::core::BrokerConsumer;
use common::models::Task;
use futures::future::BoxFuture;
use tracing::info;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NewTaskController {
    consumer: Arc<dyn BrokerConsumer<Task>>,
    worker_kind_repository: Arc<PgWorkerKindRepository>,
    task_repository: Arc<PgTaskRepository>,
}

impl NewTaskController {
    pub async fn new(
        consumer: Arc<dyn BrokerConsumer<Task>>,
        worker_kind_repository: Arc<PgWorkerKindRepository>,
        task_repository: Arc<PgTaskRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            consumer,
            worker_kind_repository,
            task_repository,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handler = Box::new(
            |task: Task| -> BoxFuture<'_, Result<(), Box<dyn std::error::Error>>> {
                Box::pin(async move {
                    info!("Received task input: {:?}", task);
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
