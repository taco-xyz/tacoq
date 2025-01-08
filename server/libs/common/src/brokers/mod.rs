pub mod core;
pub mod rabbit;
pub mod testing;

use core::BrokerCore;
use rabbit::RabbitBroker;
use uuid::Uuid;

use std::sync::Arc;

use crate::{TaskInstance, Worker};

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
    pub workers: Vec<Worker>,
    pub workers_index: usize,

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
            workers: Vec::new(),
            workers_index: 0,
            submission_exchange: Self::SUBMISSION_EXCHANGE,
        })
    }

    pub async fn register_worker(
        &mut self,
        worker: Worker,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a unique queue for this worker using its ID
        let worker_queue = worker.id.to_string();

        self.broker
            .register_queue(Self::SUBMISSION_EXCHANGE, &worker_queue, &worker_queue)
            .await?;

        self.workers.push(worker);
        Ok(())
    }

    pub async fn remove_worker(
        &mut self,
        worker_id: &Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let index: usize = self
            .workers
            .iter()
            .position(|worker| worker.id == *worker_id)
            .unwrap();
        self.workers.remove(index);
        self.broker.delete_queue(&worker_id.to_string()).await?;

        Ok(())
    }

    pub async fn publish(
        &mut self,
        task: &TaskInstance,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let worker = (0..self.workers.len())
            // Cycle the workers list in a round robin fashion
            .map(|_| {
                let cur_worker = &self.workers[self.workers_index];
                self.workers_index = (self.workers_index + 1) % self.workers.len();
                cur_worker
            })
            // Find the first worker that can handle the task
            .find(|cur_worker| cur_worker.can_handle(task))
            .ok_or("No available worker")?;

        // Convert input data to bytes
        let payload = serde_json::to_vec(&task.input_data)?;

        // Use task type as exchange, worker ID as routing key
        self.broker
            .publish_message(
                Self::SUBMISSION_EXCHANGE,
                &worker.id.to_string(),
                &payload,
                &task.id.to_string(),
                &task.task_kind.name,
            )
            .await?;

        Ok(worker.id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::TaskKind;
    use crate::TaskStatus;
    use testing::{get_mock_broker, setup_task_kinds, setup_tasks, setup_workers};
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_broker_new() {
        let broker = get_mock_broker();
        assert_eq!(broker.uri, "mock");
        assert_eq!(broker.workers.len(), 0);
        assert_eq!(broker.workers_index, 0);
    }

    #[tokio::test]
    async fn test_broker_register_worker() {
        let mut broker = get_mock_broker();
        let workers = setup_workers(setup_task_kinds());

        for worker in workers {
            broker.register_worker(worker).await.unwrap();
        }

        assert_eq!(broker.workers.len(), 3);
    }

    #[tokio::test]
    async fn test_broker_remove_worker() {
        let mut broker = get_mock_broker();
        let workers = setup_workers(setup_task_kinds());

        for worker in workers.clone() {
            broker.register_worker(worker).await.unwrap();
        }

        broker.remove_worker(&workers[0].id).await.unwrap();
        assert_eq!(broker.workers.len(), 2);
    }

    #[tokio::test]
    async fn test_broker_publish() {
        let task_kinds = setup_task_kinds();
        let workers = setup_workers(task_kinds.clone());
        let tasks = setup_tasks(task_kinds.clone());

        let mut broker = get_mock_broker();

        for worker in workers.clone() {
            broker.register_worker(worker).await.unwrap();
        }

        for task in tasks {
            broker.publish(&task).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_no_available_worker() {
        let mut broker = get_mock_broker();

        let workers = setup_workers(setup_task_kinds());

        for worker in workers.clone() {
            broker.register_worker(worker).await.unwrap();
        }

        let task = TaskInstance {
            id: Uuid::new_v4(),
            task_kind: TaskKind::new("task3".to_string()),
            input_data: Some(serde_json::json!({"key": "value"})),
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            assigned_to: None,
            result: None,
        };

        let result = broker.publish(&task).await;
        assert!(result.is_err());
    }
}
