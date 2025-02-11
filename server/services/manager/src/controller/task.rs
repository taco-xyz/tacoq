use crate::repo::worker_kind_repo::PgWorkerKindRepository;
use crate::repo::{
    PgTaskRepository, PgWorkerRepository, TaskRepository, WorkerKindRepository, WorkerRepository,
};
use common::brokers::core::BrokerConsumer;
use common::models::Task;
use tracing::info;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskController {
    consumer: Arc<dyn BrokerConsumer<Task>>,
    worker_repository: PgWorkerRepository,
    worker_kind_repository: PgWorkerKindRepository,
    task_repository: PgTaskRepository,
}

impl TaskController {
    pub async fn new(
        consumer: Arc<dyn BrokerConsumer<Task>>,
        worker_repository: PgWorkerRepository,
        worker_kind_repository: PgWorkerKindRepository,
        task_repository: PgTaskRepository,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            consumer,
            worker_repository,
            worker_kind_repository,
            task_repository,
        })
    }

    async fn consume_task(&self, task: Task) -> Result<(), Box<dyn std::error::Error>> {
        // I get/create the worker kind -> necessary for relationship
        let worker_kind = self
            .worker_kind_repository
            .get_or_create_worker_kind(&task.worker_kind)
            .await?;

        // I get a task -> save it into the database
        let task = self.task_repository.update_task(&task).await?;

        // I check the worker and add it to the DB if it exists as well
        if let Some(assigned_to) = task.assigned_to {
            self.worker_repository
                .update_worker(assigned_to, &worker_kind.name)
                .await?;
        }

        info!("Received task input: {:?}", task);
        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        // This clone is needed so the self object can live inside the closure due to move
        let this = self.clone();

        self.consumer
            .consume_messages(Box::new(move |task| {
                // Same thing here, we need to clone the self object so it can live inside the closure for 'static lifetime
                let this = this.clone();
                Box::pin(async move { this.consume_task(task).await })
            }))
            .await
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.shutdown().await
    }
}
