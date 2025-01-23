use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

// Task Type

/// A task type is a type of task that can be executed by a worker.
/// It is defined by its name and its input data schema.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskKind {
    pub id: Uuid,
    pub name: String,
    pub worker_kind_name: String,
    pub created_at: OffsetDateTime,
}

impl TaskKind {
    pub fn new(name: &str, worker_kind_name: &str) -> TaskKind {
        TaskKind {
            id: Uuid::new_v4(),
            name: name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
