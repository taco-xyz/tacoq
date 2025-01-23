use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub worker_kind_name: String,
    #[serde(
        serialize_with = "crate::models::serialize_datetime",
        deserialize_with = "crate::models::deserialize_datetime"
    )]
    pub created_at: OffsetDateTime,
}

impl Worker {
    pub fn new(name: &str, worker_kind_name: &str) -> Self {
        Worker {
            id: Uuid::new_v4(),
            name: name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct WorkerHeartbeat {
    pub worker_id: Uuid,
    pub heartbeat_time: OffsetDateTime,
    pub created_at: OffsetDateTime,
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
