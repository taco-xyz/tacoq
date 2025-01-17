use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

use time::OffsetDateTime;

use super::TaskInstance;
use crate::models::TaskKind;

/// A worker that can execute tasks after receiving them.
/// We know that it can receive those tasks from its list of capabilities.
/// A worker must register itself with its capabilities to be able to receive tasks.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    #[serde(
        serialize_with = "crate::models::serialize_datetime",
        deserialize_with = "crate::models::deserialize_datetime"
    )]
    pub registered_at: OffsetDateTime,
    pub task_kind: Vec<TaskKind>,
    pub active: bool,
}

impl Worker {
    pub fn new(name: String, task_kind: Vec<TaskKind>) -> Self {
        Worker {
            id: Uuid::new_v4(),
            name,
            registered_at: OffsetDateTime::now_utc(),
            task_kind,
            active: true,
        }
    }

    pub fn can_handle(&self, task: &TaskInstance) -> bool {
        self.task_kind
            .iter()
            .any(|kind| kind.name == task.task_kind.name)
    }
}

#[cfg(test)]
mod test {
    use super::{TaskKind, Worker};
    use crate::TaskInstance;
    use sqlx::types::Uuid;
    use time::OffsetDateTime;

    #[test]
    fn test_worker_can_handle() {
        let task_kind1 = TaskKind::new("task1".to_string());
        let task_kind2 = TaskKind::new("task2".to_string());
        let worker = Worker::new("worker1".to_string(), vec![task_kind1.clone()]);

        let task1 = TaskInstance {
            id: Uuid::new_v4(),
            task_kind: task_kind1.clone(),
            status: crate::TaskStatus::Queued,
            created_at: OffsetDateTime::now_utc(),
            input_data: None,
            assigned_to: None,
            result: None,
        };
        let task2 = TaskInstance {
            id: Uuid::new_v4(),
            task_kind: task_kind2.clone(),
            status: crate::TaskStatus::Queued,
            created_at: OffsetDateTime::now_utc(),
            input_data: None,
            assigned_to: None,
            result: None,
        };

        assert!(worker.can_handle(&task1));
        assert!(!worker.can_handle(&task2));
    }
}
