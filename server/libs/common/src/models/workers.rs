use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

use time::OffsetDateTime;

use crate::models::WorkerKind;

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
    pub worker_kind: WorkerKind,
    pub active: bool,
}

impl Worker {
    pub fn new(name: String, worker_kind: WorkerKind) -> Self {
        Worker {
            id: Uuid::new_v4(),
            name,
            registered_at: OffsetDateTime::now_utc(),
            worker_kind,
            active: true,
        }
    }
}
