use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkerKind {
    pub name: String,
    pub routing_key: String,
    pub queue_name: String,
    pub created_at: OffsetDateTime,
}

impl WorkerKind {
    pub fn new(name: String, routing_key: String, queue_name: String) -> Self {
        WorkerKind {
            name,
            routing_key,
            queue_name,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
