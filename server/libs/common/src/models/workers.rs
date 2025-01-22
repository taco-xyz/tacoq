use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

use time::OffsetDateTime;

/// A worker that can execute tasks after receiving them.
/// We know that it can receive those tasks from its list of capabilities.
/// A worker must register itself with its capabilities to be able to receive tasks.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub worker_kind_name: String,
    #[serde(
        serialize_with = "crate::models::serialize_datetime",
        deserialize_with = "crate::models::deserialize_datetime"
    )]
    pub registered_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkerHeartbeat {
    pub worker_id: Uuid,
    pub heartbeat_time: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Worker {
    pub fn new(name: String, worker_kind_name: String) -> Self {
        Worker {
            id: Uuid::new_v4(),
            name,
            worker_kind_name,
            registered_at: OffsetDateTime::now_utc(),
        }
    }
}

impl WorkerHeartbeat {
    pub fn new(worker_id: Uuid) -> Self {
        WorkerHeartbeat {
            worker_id,
            heartbeat_time: OffsetDateTime::now_utc(),
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
