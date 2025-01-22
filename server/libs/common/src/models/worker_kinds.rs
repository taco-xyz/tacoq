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
