use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkerKind {
    pub name: String,
    pub routing_key: String,
    pub queue_name: String,
    pub created_at: DateTime<Utc>,
}

impl WorkerKind {
    pub fn new(name: &str, routing_key: &str, queue_name: &str) -> Self {
        WorkerKind {
            name: name.to_string(),
            routing_key: routing_key.to_string(),
            queue_name: queue_name.to_string(),
            created_at: Utc::now(),
        }
    }
}
