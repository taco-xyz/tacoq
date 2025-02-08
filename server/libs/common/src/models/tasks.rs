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
#[derive(Default, Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(default)]
pub struct Task {
    pub id: Uuid,
    pub task_kind_name: String,

    // Task data
    pub input_data: Option<serde_json::Value>,
    pub output_data: Option<serde_json::Value>,
    pub is_error: i32,

    // Task status
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    pub ttl: Option<DateTime<Utc>>, // Time to live only enabled after it has been completed

    // Relations
    pub worker_kind_name: String,
    pub assigned_to: Option<Uuid>, // worker that it is assigned to

    // Metadata
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(
        id: Option<Uuid>,
        task_kind_name: &str,
        worker_kind_name: &str,
        input_data: Option<serde_json::Value>,
        output_data: Option<serde_json::Value>,
        is_error: Option<bool>,
        created_at: DateTime<Utc>,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        assigned_to: Option<Uuid>,
        ttl: Option<DateTime<Utc>>,
    ) -> Self {
        Task {
            id: id.unwrap_or(Uuid::new_v4()),
            task_kind_name: task_kind_name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            input_data,
            output_data,
            is_error: if is_error.is_some() {
                if is_error.unwrap() {
                    1
                } else {
                    0
                }
            } else {
                0
            },
            started_at,
            completed_at,
            assigned_to,
            created_at,
            updated_at: Utc::now(),
            ttl,
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

    pub fn status(&self) -> TaskStatus {
        if self.completed_at.is_some() {
            TaskStatus::Completed
        } else if self.started_at.is_some() {
            TaskStatus::Processing
        } else {
            TaskStatus::Pending
        }
    }
}
