use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum_macros::{Display, EnumString};
use utoipa::ToSchema;
use uuid::Uuid;

// Custom serializer function
fn serialize_bytes<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match bytes {
        Some(data) => {
            // Always use base64 for consistency
            let encoded = STANDARD.encode(data);
            serializer.serialize_str(&encoded)
        }
        None => serializer.serialize_none(),
    }
}

// Custom deserializer function
fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    if let Some(input) = Option::<String>::deserialize(deserializer)? {
        // First try to decode as base64
        if let Ok(decoded) = STANDARD.decode(&input) {
            return Ok(Some(decoded));
        }
        // If not base64, treat as JSON string and convert to bytes
        Ok(Some(input.into_bytes()))
    } else {
        Ok(None)
    }
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Try parsing with different formats
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try parsing microseconds format like "2025-02-14T13:35:07.365122"
    if let Ok(dt) = DateTime::parse_from_str(&format!("{}+00:00", s), "%Y-%m-%dT%H:%M:%S%.f%:z") {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try parsing without fractional seconds
    if let Ok(dt) = DateTime::parse_from_str(&format!("{}+00:00", s), "%Y-%m-%dT%H:%M:%S%:z") {
        return Ok(dt.with_timezone(&Utc));
    }

    Err(serde::de::Error::custom(format!(
        "Unable to parse datetime: {}",
        s
    )))
}

// Task status enum
/// # Possible Status:
/// * `Pending`: Task is created but not yet assigned
/// * `Processing`: Task has been assigned to a worker and sent to a queue
/// * `Completed`: Task completed successfully or not
#[derive(Display, EnumString, Debug, Serialize, Deserialize, PartialEq, ToSchema, Clone)]
pub enum TaskStatus {
    #[strum(serialize = "pending")]
    Pending, // Task is created but not yet assigned
    #[strum(serialize = "processing")]
    Processing, // Task has been assigned to a worker and sent to a queue
    #[strum(serialize = "completed")]
    Completed, // Task completed successfully or not
}

// Task

/// Tasks are sent to workers to be executed with a specific payload.
/// Workers are eligble for receiving certain tasks depending on their
/// list of capabilities.
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    #[sqlx(rename = "task_kind_name")]
    pub task_kind: String,

    // Task data
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    pub input_data: Option<Vec<u8>>, // byte array
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes",
        alias = "result"
    )]
    pub output_data: Option<Vec<u8>>, // byte array
    pub is_error: i32,

    pub status: TaskStatus,
    pub priority: i32,

    // Relations
    #[sqlx(rename = "worker_kind_name")]
    pub worker_kind: String,
    pub assigned_to: Option<Uuid>, // worker that it is assigned to

    // Task status
    #[serde(skip)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub ttl: Option<DateTime<Utc>>, // Time to live only enabled after it has been completed

    // Metadata
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(
        id: Option<Uuid>,
        task_kind_name: &str,
        worker_kind_name: &str,
        input_data: Option<Vec<u8>>,
        output_data: Option<Vec<u8>>,
        is_error: Option<bool>,
        status: TaskStatus,
        priority: i32,
        created_at: DateTime<Utc>,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        assigned_to: Option<Uuid>,
        ttl: Option<DateTime<Utc>>,
    ) -> Self {
        Task {
            id: id.unwrap_or_else(Uuid::new_v4),
            task_kind: task_kind_name.to_string(),
            input_data,
            output_data,
            is_error: is_error.map_or(0, |e| if e { 1 } else { 0 }),
            status,
            priority,
            worker_kind: worker_kind_name.to_string(),
            assigned_to,
            started_at,
            completed_at,
            ttl,
            created_at,
            updated_at: Utc::now(),
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
        self.status = status;
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
