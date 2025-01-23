use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres};
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
    pub assigned_to: Option<Uuid>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(task_kind_name: &str, input_data: Option<serde_json::Value>) -> Self {
        Task {
            id: Uuid::new_v4(),
            task_kind_name: task_kind_name.to_string(),
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

    pub async fn save<'e, E>(&self, executor: E) -> Result<Task, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO tasks (
                id, task_kind_name, input_data, started_at, completed_at, ttl, assigned_to,
                is_error, output_data, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                input_data = EXCLUDED.input_data,
                started_at = EXCLUDED.started_at,
                completed_at = EXCLUDED.completed_at,
                ttl = EXCLUDED.ttl,
                assigned_to = EXCLUDED.assigned_to,
                is_error = EXCLUDED.is_error,
                output_data = EXCLUDED.output_data,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(self.id)
        .bind(&self.task_kind_name)
        .bind(&self.input_data)
        .bind(self.started_at)
        .bind(self.completed_at)
        .bind(self.ttl)
        .bind(self.assigned_to)
        .bind(self.is_error)
        .bind(&self.output_data)
        .bind(self.created_at)
        .bind(self.updated_at)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &Uuid) -> Result<Task, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
    }
}
