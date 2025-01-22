use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

// Task Type

/// A task type is a type of task that can be executed by a worker.
/// It is defined by its name and its input data schema.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskKind {
    pub name: String,
    pub worker_kind_name: String,
    pub created_at: OffsetDateTime,
}

impl TaskKind {
    pub fn new(name: String, worker_kind_name: String) -> Self {
        TaskKind {
            name,
            worker_kind_name,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
