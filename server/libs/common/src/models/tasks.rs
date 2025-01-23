use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum_macros::Display;
use time::{Duration, OffsetDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

// Task status enum
/// # Possible Status:
/// * `Pending`: Task is created but not yet assigned
/// * `Processing`: Task has been assigned to a worker and sent to a queue
/// * `Completed`: Task completed sucessfully or not
#[derive(Display, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum TaskStatus {
    Pending,    // Task is created but not yet assigned
    Processing, // Task has been assigned to a worker and sent to a queue
    Completed,  // Task completed sucessfully or not
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        s.to_lowercase()
            .as_str()
            .try_into()
            .unwrap_or_else(|_| panic!("Invalid task status: {}", s))
    }
}

impl TryFrom<&str> for TaskStatus {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "processing" => Ok(TaskStatus::Processing),
            "completed" => Ok(TaskStatus::Completed),
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }
}

impl From<TaskStatus> for String {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => "pending",
            TaskStatus::Processing => "processing",
            TaskStatus::Completed => "completed",
        }
        .to_string()
    }
}

// Task

/// Tasks are sent to workers to be executed with a specific payload.
/// Workers are eligble for receiving certain tasks depending on their
/// list of capabilities.
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,

    // Task data
    pub input_data: Option<serde_json::Value>,
    pub output_data: Option<serde_json::Value>,
    pub is_error: i8,

    // Task status
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub ttl: OffsetDateTime,

    // Relations
    pub task_kind_id: Uuid,
    pub assigned_to: Option<Uuid>,

    // Metadata
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Task {
    pub fn new(task_kind_id: Uuid, input_data: Option<serde_json::Value>) -> Self {
        Task {
            id: Uuid::new_v4(),
            input_data,
            output_data: None,
            is_error: 0,
            started_at: None,
            completed_at: None,
            task_kind_id,
            assigned_to: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            ttl: OffsetDateTime::now_utc() + Duration::days(7),
        }
    }

    pub fn mark_processing(&mut self, worker_id: Uuid) {
        self.assigned_to = Some(worker_id);
        self.started_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn mark_completed(&mut self) {
        self.completed_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_result(&mut self, worker_id: Uuid, output: serde_json::Value) {
        self.assigned_to = Some(worker_id);
        self.output_data = Some(output);
        self.completed_at = Some(OffsetDateTime::now_utc());
        self.is_error = 0;
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn set_error(&mut self, worker_id: Uuid, error: serde_json::Value) {
        self.assigned_to = Some(worker_id);
        self.output_data = Some(error);
        self.completed_at = Some(OffsetDateTime::now_utc());
        self.is_error = 1;
        self.updated_at = OffsetDateTime::now_utc();
    }
}
