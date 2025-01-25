use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum_macros::{Display, EnumString};
use utoipa::ToSchema;
use uuid::Uuid;

// Task status enum
/// # Possible Status:
/// * `Pending`: Task is created but not yet assigned
/// * `Processing`: Task has been assigned to a worker and sent to a queue
/// * `Completed`: Task completed sucessfully or not
#[derive(Display, EnumString, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum TaskStatus {
    #[strum(serialize = "pending")]
    Pending, // Task is created but not yet assigned
    #[strum(serialize = "processing")]
    Processing, // Task has been assigned to a worker and sent to a queue
    #[strum(serialize = "completed")]
    Completed, // Task completed sucessfully or not
}

// Task

/// Tasks are sent to workers to be executed with a specific payload.
/// Workers are eligble for receiving certain tasks depending on their
/// list of capabilities.
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub task_kind_name: String,

    // Task data
    pub input_data: Option<serde_json::Value>,
    pub output_data: Option<serde_json::Value>,
    pub is_error: i32,

    // Task status
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    pub ttl: Option<DateTime<Utc>>, // Time to live only enabled after it has been completed

    // Relations
    pub worker_kind_name: String,
    pub assigned_to: Option<Uuid>, // worker that it is assigned to

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(
        task_kind_name: &str,
        worker_kind_name: &str,
        input_data: Option<serde_json::Value>,
    ) -> Self {
        Task {
            id: Uuid::new_v4(),
            task_kind_name: task_kind_name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            input_data,
            output_data: None,
            is_error: 0,
            started_at: None,
            completed_at: None,
            assigned_to: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ttl: None,
        }
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        match status {
            TaskStatus::Pending => {
                self.started_at = None;
                self.completed_at = None;
                self.ttl = None;
                self.assigned_to = None;
            }
            TaskStatus::Processing => {
                self.started_at = Some(Utc::now());
                self.completed_at = None;
                self.ttl = None;
            }
            TaskStatus::Completed => {
                self.completed_at = Some(Utc::now());
                self.ttl = Some(Utc::now() + Duration::days(7));
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn mark_processing(&mut self, worker_id: Uuid) {
        self.assigned_to = Some(worker_id);
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn mark_completed(&mut self, output_data: serde_json::Value, is_error: bool) {
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.ttl = Some(Utc::now() + Duration::days(7));
        self.output_data = Some(output_data);
        self.is_error = if is_error { 1 } else { 0 };
    }
}
