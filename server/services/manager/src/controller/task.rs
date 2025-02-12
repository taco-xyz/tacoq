use crate::repo::worker_kind_repo::PgWorkerKindRepository;
use crate::repo::{
    PgTaskRepository, PgWorkerRepository, TaskRepository, WorkerKindRepository, WorkerRepository,
};
use common::brokers::core::BrokerConsumer;
use common::models::Task;
use tracing::{error, info, warn};

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

    async fn consume_task(&self, task: Task) {
        info!("Starting to process task: {:?}", task);

        let kind = match self
            .worker_kind_repository
            .get_or_create_worker_kind(&task.worker_kind)
            .await
        {
            Ok(kind) => kind,
            Err(e) => {
                error!("Failed to process worker kind: {}", e);
                return;
            }
        };
        info!("Worker kind processed successfully: {:?}", kind);

        let task = match self.task_repository.update_task(&task).await {
            Ok(task) => task,
            Err(e) => {
                error!("Failed to update task: {}", e);
                return;
            }
        };
        info!("Task updated successfully: {:?}", task);

        if let Some(assigned_to) = task.assigned_to {
            match self
                .worker_repository
                .update_worker(assigned_to, &kind.name)
                .await
            {
                Ok(_) => info!("Worker updated successfully: {}", assigned_to),
                Err(e) => warn!("Failed to update worker {}: {}", assigned_to, e),
            }
        }

        info!("Task processing completed successfully");
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        // This clone is needed so the self object can live inside the closure due to move
        let this = self.clone();

        self.consumer
            .consume_messages(Box::new(move |task| {
                // Same thing here, we need to clone the self object so it can live inside the closure for 'static lifetime
                let this = this.clone();
                Box::pin(async move {
                    this.consume_task(task).await;
                })
            }))
            .await
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.shutdown().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::PgRepositoryCore;
    use crate::testing::{controller::TestController, test::init_test_logger};
    use chrono::Utc;
    use common::models::Task;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    fn get_test_task(worker_kind: &str) -> Task {
        Task::new(
            Some(Uuid::new_v4()),
            "test_task",
            worker_kind,
            None,
            None,
            None,
            Utc::now(),
            None,
            None,
            None,
            None,
        )
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_successful_task_processing(db_pools: PgPool) {
        let controller = TestController::new(db_pools.clone()).await;

        let test_task = get_test_task("test_worker_kind");
        let task_id = test_task.id;

        controller.consume(test_task).await;

        // Verify the task was processed
        let core = PgRepositoryCore::new(db_pools);
        let task_repo = PgTaskRepository::new(core);
        let stored_task = task_repo.get_task_by_id(&task_id).await.unwrap().unwrap();

        assert_eq!(stored_task.id, task_id);
        assert_eq!(stored_task.worker_kind, "test_worker_kind");
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_task_with_assigned_worker(db_pools: PgPool) {
        let controller = TestController::new(db_pools.clone()).await;

        let worker_id = Uuid::new_v4();
        let mut test_task = get_test_task("test_worker_kind");
        test_task.assigned_to = Some(worker_id);

        controller.consume(test_task).await;

        // Verify the worker was updated
        let core = PgRepositoryCore::new(db_pools);
        let worker_repo = PgWorkerRepository::new(core);
        let worker = worker_repo
            .get_worker_by_id(&worker_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(worker.id, worker_id);
        assert_eq!(worker.kind, "test_worker_kind");
    }
}
