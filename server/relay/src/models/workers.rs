use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct Worker {
    pub name: String,
    pub worker_kind_name: String,
    pub registered_at: DateTime<Utc>,
}

impl Worker {
    pub fn new(name: &str, worker_kind_name: &str) -> Self {
        Worker {
            name: name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            registered_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct WorkerHeartbeat {
    pub worker_name: String,
    pub heartbeat_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl WorkerHeartbeat {
    pub fn new(worker_name: &str) -> Self {
        WorkerHeartbeat {
            worker_name: worker_name.to_string(),
            heartbeat_time: Utc::now(),
            created_at: Utc::now(),
        }
    }
}
