use crate::{TaskInstance, TaskKind, TaskStatus, Worker};
use time::OffsetDateTime;
use uuid::Uuid;

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
