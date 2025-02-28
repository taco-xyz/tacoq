use std::collections::HashMap;
use std::fmt::Error;

use chrono::{DateTime, Duration, Utc};
use opentelemetry::propagation::{Extractor, TextMapPropagator};
use opentelemetry::Context;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
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
        Some(data) => serializer.serialize_str(&String::from_utf8_lossy(data)),
        None => serializer.serialize_none(),
    }
}

// Custom deserializer function
fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let input: Option<String> = Option::deserialize(deserializer)?;
    match input {
        Some(bytes) => Ok(Some(bytes.into_bytes())),
        None => Ok(None),
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

fn deserialize_timestamp_optional<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_timestamp(deserializer).map(Some).or(Ok(None))
}

// Task status enum
/// # Possible Status:
/// * `Pending`: Task is created but not yet assigned
/// * `Processing`: Task has been assigned to a worker and sent to a queue
/// * `Completed`: Task completed successfully or not
#[derive(Display, EnumString, Debug, Serialize, Deserialize, PartialEq, ToSchema, Clone)]
pub enum TaskStatus {
    #[strum(serialize = "pending")]
    #[serde(rename = "pending")]
    Pending, // Task is created but not yet assigned
    #[strum(serialize = "processing")]
    #[serde(rename = "processing")]
    Processing, // Task has been assigned to a worker and sent to a queue
    #[strum(serialize = "completed")]
    #[serde(rename = "completed")]
    Completed, // Task completed successfully or not
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(TaskStatus::Pending)
    }
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
        deserialize_with = "deserialize_bytes"
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
    #[serde(deserialize_with = "deserialize_timestamp_optional")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_timestamp_optional")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub ttl: Option<DateTime<Utc>>, // Time to live only enabled after it has been completed

    // Metadata
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,

    // OpenTelemetry context carrier
    pub otel_ctx_carrier: Option<JsonValue>,
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
        otel_ctx_carrier: Option<JsonValue>,
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
            otel_ctx_carrier,
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

    pub fn context(&self) -> Context {
        let carrier_value = self.otel_ctx_carrier.clone();
        match carrier_value {
            Some(carrier) => extract_context(&carrier).unwrap(),
            None => Context::new(),
        }
    }
}

// Context Extraction (this was a motherfucker)

struct HashMapExtractor<'a>(&'a std::collections::HashMap<String, String>);

impl<'a> Extractor for HashMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Removes all non-string values from the map. Basically ensures that
/// the map is a valid OpenTelemetry context carrier.
fn strip_map(map: &serde_json::Map<String, JsonValue>) -> HashMap<String, String> {
    let hashmap: HashMap<String, String> = map
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
        .collect();
    hashmap
}

fn extract_context(carrier: &JsonValue) -> Result<Context, Error> {
    match carrier {
        JsonValue::Object(map) => {
            let propagator = TraceContextPropagator::new();
            let otel_cx = propagator.extract(&HashMapExtractor(&strip_map(map)));
            Ok(otel_cx)
        }
        _ => Err(Error),
    }
}
