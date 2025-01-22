pub mod task_instance;
pub mod task_result;

use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::BrokerConsumer;
use std::sync::Arc;
use tracing::info;

use common::models::{TaskInstance, TaskResult};

pub struct Controllers {
    pub task_input: Arc<task_instance::TaskInstanceController>,
    pub task_result: Arc<task_result::TaskResultController>,
}

impl Controllers {
    pub async fn new(
        task_instance_broker: Arc<dyn BrokerConsumer<TaskInstance>>,
        task_result_broker: Arc<dyn BrokerConsumer<TaskResult>>,
        task_repo: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let task_input = Arc::new(
            task_instance::TaskInstanceController::new(task_instance_broker, task_repo.clone())
                .await?,
        );
        let task_result =
            Arc::new(task_result::TaskResultController::new(task_result_broker, task_repo).await?);

        Ok(Self {
            task_input,
            task_result,
        })
    }

    pub async fn run(&self) -> (tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>) {
        let input_controller = self.task_input.clone();
        let result_controller = self.task_result.clone();

        let input_handle = tokio::spawn(async move {
            input_controller
                .run()
                .await
                .expect("Task input controller failed");
        });

        let result_handle = tokio::spawn(async move {
            result_controller
                .run()
                .await
                .expect("Task result controller failed");
        });

        (input_handle, result_handle)
    }

    pub async fn cleanup(&self) {
        if let Err(e) = self.task_input.cleanup().await {
            info!("Error cleaning up task input controller: {}", e);
        }
        if let Err(e) = self.task_result.cleanup().await {
            info!("Error cleaning up task result controller: {}", e);
        }
    }
}
