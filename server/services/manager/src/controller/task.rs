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
    use crate::{
        repo::{PgRepositoryCore, PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository},
        testing::test::init_test_logger,
    };
    use common::brokers::core::MockBrokerConsumer;
    use sqlx::{types::chrono::Utc, PgPool};
    use uuid::Uuid;

    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    fn get_test_task() -> Task {
        Task::new(
            Some(Uuid::new_v4()),
            "TestTaskKind",
            "TestWorkerKind",
            Some(serde_json::json!({})),
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
    async fn test_successful_task_consumption(pool: PgPool) {
        let core = PgRepositoryCore::new(pool.clone());
        let task_repo = PgTaskRepository::new(core.clone());
        let worker_repo = PgWorkerRepository::new(core.clone());
        let worker_kind_repo = PgWorkerKindRepository::new(core);

        let mut mock_consumer = MockBrokerConsumer::<Task>::new();
        let test_task = get_test_task();
        let test_task_clone = test_task.clone();

        mock_consumer
            .expect_consume_messages()
            .times(1)
            .returning(move |handler| {
                let test_task = test_task_clone.clone();
                tokio::spawn(async move {
                    handler(test_task).await;
                });
                Ok(())
            });

        mock_consumer
            .expect_shutdown()
            .times(1)
            .returning(|| Ok(()));

        let controller = Arc::new(
            TaskController::new(
                Arc::new(mock_consumer),
                worker_repo,
                worker_kind_repo,
                task_repo,
            )
            .await
            .unwrap(),
        );

        let handle = tokio::spawn({
            let controller = controller.clone();
            async move {
                controller.run().await.unwrap();
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        controller.shutdown().await.unwrap();
        handle.await.unwrap();

        let processed_task = controller
            .task_repository
            .get_task_by_id(&test_task.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(processed_task.worker_kind, test_task.worker_kind);
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_task_consumption_with_invalid_worker_kind(pool: PgPool) {
        let core = PgRepositoryCore::new(pool.clone());
        let task_repo = PgTaskRepository::new(core.clone());
        let worker_repo = PgWorkerRepository::new(core.clone());
        let worker_kind_repo = PgWorkerKindRepository::new(core);

        let mut mock_consumer = MockBrokerConsumer::<Task>::new();
        let mut test_task = get_test_task();
        let test_task_clone = test_task.clone();
        test_task.worker_kind = "".to_string();

        mock_consumer
            .expect_consume_messages()
            .times(1)
            .returning(move |handler| {
                let test_task = test_task_clone.clone();
                tokio::spawn(async move {
                    handler(test_task).await;
                });
                Ok(())
            });

        mock_consumer
            .expect_shutdown()
            .times(1)
            .returning(|| Ok(()));

        let controller = Arc::new(
            TaskController::new(
                Arc::new(mock_consumer),
                worker_repo,
                worker_kind_repo,
                task_repo,
            )
            .await
            .unwrap(),
        );

        let handle = tokio::spawn({
            let controller = controller.clone();
            async move {
                controller.run().await.unwrap();
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        controller.shutdown().await.unwrap();
        handle.await.unwrap();

        let result = controller
            .task_repository
            .get_task_by_id(&test_task.id)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_task_consumption_with_missing_worker(pool: PgPool) {
        let core = PgRepositoryCore::new(pool.clone());
        let task_repo = PgTaskRepository::new(core.clone());
        let worker_repo = PgWorkerRepository::new(core.clone());
        let worker_kind_repo = PgWorkerKindRepository::new(core);

        let mut mock_consumer = MockBrokerConsumer::<Task>::new();
        let mut test_task = get_test_task();
        let test_task_clone = test_task.clone();
        test_task.assigned_to = None;

        mock_consumer
            .expect_consume_messages()
            .times(1)
            .returning(move |handler| {
                let test_task = test_task_clone.clone();
                tokio::spawn(async move {
                    handler(test_task).await;
                });
                Ok(())
            });

        mock_consumer
            .expect_shutdown()
            .times(1)
            .returning(|| Ok(()));

        let controller = Arc::new(
            TaskController::new(
                Arc::new(mock_consumer),
                worker_repo,
                worker_kind_repo,
                task_repo,
            )
            .await
            .unwrap(),
        );

        let handle = tokio::spawn({
            let controller = controller.clone();
            async move {
                controller.run().await.unwrap();
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        controller.shutdown().await.unwrap();
        handle.await.unwrap();

        let processed_task = controller
            .task_repository
            .get_task_by_id(&test_task.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(processed_task.worker_kind, test_task.worker_kind);
        assert!(processed_task.assigned_to.is_none());
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_multiple_task_consumption(pool: PgPool) {
        let core = PgRepositoryCore::new(pool.clone());
        let task_repo = PgTaskRepository::new(core.clone());
        let worker_repo = PgWorkerRepository::new(core.clone());
        let worker_kind_repo = PgWorkerKindRepository::new(core);

        let mut mock_consumer = MockBrokerConsumer::<Task>::new();
        let task1 = get_test_task();
        let task2 = get_test_task();
        let task3 = get_test_task();

        let tasks = vec![task1.clone(), task2.clone(), task3.clone()];

        mock_consumer
            .expect_consume_messages()
            .times(1)
            .returning(move |handler| {
                let tasks = tasks.clone();
                tokio::spawn(async move {
                    for task in tasks {
                        handler(task).await;
                    }
                });
                Ok(())
            });

        mock_consumer
            .expect_shutdown()
            .times(1)
            .returning(|| Ok(()));

        let controller = Arc::new(
            TaskController::new(
                Arc::new(mock_consumer),
                worker_repo,
                worker_kind_repo,
                task_repo,
            )
            .await
            .unwrap(),
        );

        let handle = tokio::spawn({
            let controller = controller.clone();
            async move {
                controller.run().await.unwrap();
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        controller.shutdown().await.unwrap();
        handle.await.unwrap();

        for task in [task1, task2, task3] {
            let processed_task = controller
                .task_repository
                .get_task_by_id(&task.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(processed_task.worker_kind, task.worker_kind);
        }
    }
}
