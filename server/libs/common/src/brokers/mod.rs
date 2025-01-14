pub mod core;
pub mod rabbit;
pub mod testing;

use core::BrokerCore;
use rabbit::RabbitBroker;
use uuid::Uuid;

use std::sync::Arc;

use crate::{TaskInstance, WorkerKind};

async fn create_broker_connection(
    uri: &str,
) -> Result<Arc<dyn BrokerCore>, Box<dyn std::error::Error>> {
    let prefix = uri.split(":").collect::<Vec<&str>>()[0];

    match prefix {
        "amqp" => Ok(Arc::new(RabbitBroker::new(uri).await?)),
        _ => Err("Invalid broker URI".into()),
    }
}

#[derive(Clone)]
pub struct Broker {
    pub uri: String,
    pub broker: Arc<dyn BrokerCore>,

    // Consts
    pub submission_exchange: &'static str,
}

impl Broker {
    const SUBMISSION_EXCHANGE: &'static str = "task_submission";

    pub async fn new(uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let broker = create_broker_connection(uri).await?;
        broker.register_exchange(Self::SUBMISSION_EXCHANGE).await?;

        Ok(Self {
            uri: uri.to_string(),
            broker,
            // workers: Vec::new(),
            // workers_index: 0,
            submission_exchange: Self::SUBMISSION_EXCHANGE,
        })
    }

    // pub async fn register_worker(
    //     &mut self,
    //     worker: Worker,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     // Create a unique queue for this worker using its ID
    //     let worker_queue = worker.id.to_string();

    //     self.broker
    //         .register_queue(Self::SUBMISSION_EXCHANGE, &worker_queue, &worker_queue)
    //         .await?;

    //     self.workers.push(worker);
    //     Ok(())
    // }

    // pub async fn remove_worker(
    //     &mut self,
    //     worker_id: &Uuid,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let index: usize = self
    //         .workers
    //         .iter()
    //         .position(|worker| worker.id == *worker_id)
    //         .unwrap();
    //     self.workers.remove(index);
    //     self.broker.delete_queue(&worker_id.to_string()).await?;

    //     Ok(())
    // }

    pub async fn publish(
        &mut self,
        worker_kind: &WorkerKind,
        task: &TaskInstance,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // let worker = (0..self.workers.len())
        //     // Cycle the workers list in a round robin fashion
        //     .map(|_| {
        //         let cur_worker = &self.workers[self.workers_index];
        //         self.workers_index = (self.workers_index + 1) % self.workers.len();
        //         cur_worker
        //     })
        //     // Find the first worker that can handle the task
        //     .find(|cur_worker| cur_worker.can_handle(task))
        //     .ok_or("No available worker")?;

        // Convert input data to bytes
        let payload = serde_json::to_vec(&task.input_data)?;

        // Use task type as exchange, worker ID as routing key
        self.broker
            .publish_message(
                Self::SUBMISSION_EXCHANGE,
                &worker_kind.id.to_string(),
                &payload,
                &task.id.to_string(),
                &task.task_kind.name,
            )
            .await?;

        Ok(worker_kind.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::TaskKind;
    use crate::TaskStatus;
    use testing::{get_mock_broker, setup_task_kinds, setup_tasks, setup_worker_kinds};
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_broker_new() {
        let broker = get_mock_broker();
        assert_eq!(broker.uri, "mock");
    }

    #[tokio::test]
    async fn test_broker_publish() {
        let task_kinds = setup_task_kinds();
        let worker_kinds = setup_worker_kinds(task_kinds.clone());
        let tasks = setup_tasks(task_kinds.clone());

        let mut broker = get_mock_broker();

        for task in tasks {
            // Use the first worker kind that can handle this task
            let suitable_worker_kind = worker_kinds
                .iter()
                .find(|wk| wk.task_kinds.contains(&task.task_kind))
                .unwrap();
            broker.publish(suitable_worker_kind, &task).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_no_available_worker_kind() {
        let mut broker = get_mock_broker();
        let task_kinds = setup_task_kinds();
        let worker_kinds = setup_worker_kinds(task_kinds.clone());

        let task = TaskInstance {
            id: Uuid::new_v4(),
            task_kind: TaskKind::new("task3".to_string()),
            input_data: Some(serde_json::json!({"key": "value"})),
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            assigned_to: None,
            result: None,
        };

        // Try to publish with a worker kind that can't handle the task
        let result = broker.publish(&worker_kinds[0], &task).await;
        // The publish should succeed even if the worker kind can't handle the task
        assert!(result.is_ok());
    }
}
