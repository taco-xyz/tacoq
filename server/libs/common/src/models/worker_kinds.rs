use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{TaskInstance, TaskKind};

// Task Type

/// A task type is a type of task that can be executed by a worker.
/// It is defined by its name and its input data schema.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct WorkerKind {
    pub id: Uuid,
    pub task_kinds: Vec<TaskKind>,
}

impl WorkerKind {
    pub fn new(task_kinds: Vec<TaskKind>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_kinds,
        }
    }

    pub fn can_handle(&self, task: &TaskInstance) -> bool {
        self.task_kinds
            .iter()
            .any(|kind| kind.name == task.task_kind.name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::types::Uuid;
    use time::OffsetDateTime;

    #[test]
    fn test_worker_can_handle() {
        let task_kind1 = TaskKind::new("task1".to_string());
        let task_kind2 = TaskKind::new("task2".to_string());
        let worker_kind = WorkerKind::new(vec![task_kind1.clone()]);

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

        assert!(worker_kind.can_handle(&task1));
        assert!(!worker_kind.can_handle(&task2));
    }
}
