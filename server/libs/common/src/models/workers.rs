use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct Worker {
    pub id: Uuid,
    pub worker_kind_name: String,
    pub registered_at: DateTime<Utc>,
}

impl Worker {
    pub fn new(id: Uuid, worker_kind_name: &str) -> Self {
        Worker {
            id,
            worker_kind_name: worker_kind_name.to_string(),
            registered_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct WorkerHeartbeat {
    pub worker_id: Uuid,

    pub heartbeat_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl WorkerHeartbeat {
    pub fn new(worker_id: Uuid) -> Self {
        WorkerHeartbeat {
            worker_id,
            heartbeat_time: Utc::now(),
            created_at: Utc::now(),
        }
    }
}
