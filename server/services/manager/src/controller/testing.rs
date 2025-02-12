use crate::controller::task::TaskController;
use common::brokers::core::MockBrokerConsumer;
use common::models::Task;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::repo::{PgRepositoryCore, PgTaskRepository, PgWorkerKindRepository, PgWorkerRepository};

pub struct TestController {
    controller: Arc<TaskController>,
    consumer: Arc<MockBrokerConsumer<Task>>,
    running: Arc<Mutex<bool>>,
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl TestController {
    pub async fn new(pool: PgPool) -> Self {
        let core = PgRepositoryCore::new(pool.clone());
        let task_repository = PgTaskRepository::new(core.clone());
        let worker_repository = PgWorkerRepository::new(core.clone());
        let worker_kind_repository = PgWorkerKindRepository::new(core.clone());

        let consumer: Arc<MockBrokerConsumer<Task>> = Arc::new(MockBrokerConsumer::new());

        let controller = TaskController::new(
            consumer.clone(),
            worker_repository,
            worker_kind_repository,
            task_repository,
        )
        .await
        .unwrap();

        let controller = Arc::new(controller);

        Self {
            controller,
            consumer,
            running: Arc::new(Mutex::new(false)),
            handle: Mutex::new(None),
        }
    }

    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        if *running {
            return;
        }
        *running = true;

        let controller = self.controller.clone();
        let mut handle = self.handle.lock().await;
        *handle = Some(tokio::spawn(async move {
            controller.run().await.unwrap();
        }));
    }

    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        if !*running {
            return;
        }
        *running = false;

        self.controller.shutdown().await.unwrap();
        if let Some(handle) = self.handle.lock().await.take() {
            handle.abort();
        }
    }

    pub async fn consume(&self, task: Task) {
        let task_clone = task.clone();

        self.consumer
            .expect_consume_messages()
            .times(1)
            .withf(move |handler| {
                let task = task_clone.clone();
                tokio::spawn(async move {
                    handler(task).await;
                });
                true
            })
            .returning(|_| Ok(()));

        self.start().await;

        // Give time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

impl Drop for TestController {
    fn drop(&mut self) {
        tokio::spawn(self.stop());
    }
}
