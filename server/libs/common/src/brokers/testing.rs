use crate::brokers::core::BrokerCore;
use crate::brokers::Broker;
use crate::models::{TaskInstance, TaskKind, TaskStatus, Worker};
use async_trait::async_trait;
use sqlx::types::Uuid;
use std::sync::Arc;
use time::OffsetDateTime;

/// Mock implementations for BrokerCore that does nothing
#[derive(Clone)]
pub struct MockBrokerCore;

impl Default for MockBrokerCore {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBrokerCore {
    pub fn new() -> Self {
        MockBrokerCore
    }
}

#[async_trait]
impl BrokerCore for MockBrokerCore {
    async fn register_exchange(&self, _: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn register_queue(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn delete_queue(&self, _: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn delete_exchange(&self, _: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn publish_message(
        &self,
        _: &str,
        _: &str,
        _: &[u8],
        _: &str,
        _: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Creates and returns a broker with a mock core
pub fn get_mock_broker() -> Broker {
    Broker {
        uri: "mock".to_string(),
        broker: Arc::new(MockBrokerCore::new()),
        workers: Vec::new(),
        workers_index: 0,
        submission_exchange: "task_submission",
    }
}

pub fn setup_task_kinds() -> Vec<TaskKind> {
    vec![
        TaskKind::new("task1".to_string()),
        TaskKind::new("task2".to_string()),
    ]
}

pub fn setup_workers(task_kinds: Vec<TaskKind>) -> Vec<Worker> {
    vec![
        Worker {
            id: Uuid::new_v4(),
            name: "worker1".to_string(),
            registered_at: OffsetDateTime::now_utc(),
            task_kind: vec![task_kinds[0].clone()],
            active: true,
        },
        Worker {
            id: Uuid::new_v4(),
            name: "worker2".to_string(),
            registered_at: OffsetDateTime::now_utc(),
            task_kind: vec![task_kinds[1].clone()],
            active: true,
        },
        Worker {
            id: Uuid::new_v4(),
            name: "worker3".to_string(),
            registered_at: OffsetDateTime::now_utc(),
            task_kind: task_kinds,
            active: true,
        },
    ]
}

pub fn setup_tasks(task_kinds: Vec<TaskKind>) -> Vec<TaskInstance> {
    vec![
        TaskInstance {
            id: Uuid::new_v4(),
            task_kind: task_kinds[0].clone(),
            input_data: Some(serde_json::json!({"key": "value"})),
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            assigned_to: None,
            result: None,
        },
        TaskInstance {
            id: Uuid::new_v4(),
            task_kind: task_kinds[1].clone(),
            input_data: Some(serde_json::json!({"key": "value"})),
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            assigned_to: None,
            result: None,
        },
        TaskInstance {
            id: Uuid::new_v4(),
            task_kind: task_kinds[1].clone(),
            input_data: Some(serde_json::json!({"key": "value"})),
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            assigned_to: None,
            result: None,
        },
    ]
}
