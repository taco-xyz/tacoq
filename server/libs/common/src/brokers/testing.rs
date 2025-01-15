use crate::brokers::core::MockBrokerCore;
use crate::brokers::Broker;
use crate::{TaskInstance, TaskStatus, Worker};
use crate::{TaskKind, WorkerKind};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

/// Creates and returns a broker with a mock core
pub fn get_mock_broker() -> Broker {
    Broker {
        url: "mock".to_string(),
        name: "mock".to_string(),
        broker: Arc::new(MockBrokerCore::new()),
        exchange: None,
        queue: None,
        shutdown: Arc::new(AtomicBool::new(false)),
    }
}

pub fn setup_task_kinds() -> Vec<TaskKind> {
    vec![
        TaskKind::new("task1".to_string()),
        TaskKind::new("task2".to_string()),
    ]
}

pub fn setup_worker_kinds(task_kinds: Vec<TaskKind>) -> Vec<WorkerKind> {
    vec![
        WorkerKind::new(vec![task_kinds[0].clone()]),
        WorkerKind::new(vec![task_kinds[1].clone()]),
        WorkerKind::new(task_kinds),
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
