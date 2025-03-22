use std::collections::HashMap;

use chrono::{DateTime, Duration, Local, NaiveDateTime};
use opentelemetry::propagation::{Extractor, TextMapPropagator};
use opentelemetry::Context;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use strum_macros::{Display, EnumString};
use thiserror::Error; // Add thiserror
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{serde_avro_datetime, serde_avro_datetime_opt, AvroSerializable};
use apache_avro::{serde_avro_bytes_opt, Schema};

/// Task-related errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("Context extraction failed")]
    ContextExtractionError,
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

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Try parsing with different formats
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.naive_local());
    }

    // Try parsing microseconds format like "2025-02-14T13:35:07.365122"
    if let Ok(dt) = DateTime::parse_from_str(&format!("{}+00:00", s), "%Y-%m-%dT%H:%M:%S%.f%:z") {
        return Ok(dt.naive_local());
    }

    // Try parsing without fractional seconds
    if let Ok(dt) = DateTime::parse_from_str(&format!("{}+00:00", s), "%Y-%m-%dT%H:%M:%S%:z") {
        return Ok(dt.naive_local());
    }

    Err(serde::de::Error::custom(format!(
        "Unable to parse datetime: {}",
        s
    )))
}

fn deserialize_timestamp_optional<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
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
    // #[serde(deserialize_with = "deserialize_bytes")]
    #[serde(with = "serde_avro_bytes_opt")]
    pub input_data: Option<Vec<u8>>, // byte array
    // #[serde(deserialize_with = "deserialize_bytes")]
    #[serde(with = "serde_avro_bytes_opt")]
    pub output_data: Option<Vec<u8>>, // byte array
    pub is_error: i32,

    pub status: TaskStatus,
    pub priority: i32,

    // Relations
    #[sqlx(rename = "worker_kind_name")]
    pub worker_kind: String,
    pub executed_by: Option<String>, // worker that it is assigned to

    // Task status
    #[serde(with = "serde_avro_datetime_opt")]
    pub started_at: Option<NaiveDateTime>,
    #[serde(with = "serde_avro_datetime_opt")]
    pub completed_at: Option<NaiveDateTime>,

    pub ttl_duration: i64, // in seconds

    // Metadata
    #[serde(with = "serde_avro_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serde_avro_datetime")]
    pub updated_at: NaiveDateTime,

    // OpenTelemetry context carrier
    pub otel_ctx_carrier: Option<JsonValue>,
}

impl Task {
    /// Creates a new task with minimal required parameters
    pub fn new(
        task_kind_name: &str,
        worker_kind_name: &str,
        priority: i32,
        ttl_duration: i64,
    ) -> Self {
        Task {
            id: Uuid::new_v4(),
            task_kind: task_kind_name.to_string(),
            input_data: None,
            output_data: None,
            is_error: 0,
            status: TaskStatus::Pending,
            priority,
            worker_kind: worker_kind_name.to_string(),
            executed_by: None,
            started_at: None,
            completed_at: None,
            ttl_duration: ttl_duration,
            otel_ctx_carrier: None,
            created_at: Local::now().naive_local(),
            updated_at: Local::now().naive_local(),
        }
    }

    /// Creates a new task with specific ID
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    /// Sets the input data
    pub fn with_input_data(mut self, input_data: Vec<u8>) -> Self {
        self.input_data = Some(input_data);
        self
    }

    /// Sets the output data
    pub fn with_output_data(mut self, output_data: Vec<u8>) -> Self {
        self.output_data = Some(output_data);
        self
    }

    /// Sets the error status
    pub fn with_error(mut self, is_error: bool) -> Self {
        self.is_error = if is_error { 1 } else { 0 };
        self
    }

    /// Sets the task status
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets the assigned worker
    pub fn executed_by(mut self, worker_name: String) -> Self {
        self.executed_by = Some(worker_name);
        self
    }

    /// Sets the OpenTelemetry context
    pub fn with_otel_context(mut self, ctx: JsonValue) -> Self {
        self.otel_ctx_carrier = Some(ctx);
        self
    }

    pub fn status(&self) -> TaskStatus {
        self.status.clone()
    }

    pub fn context(&self) -> Context {
        let carrier_value = self.otel_ctx_carrier.clone();
        match carrier_value {
            Some(carrier) => extract_context(&carrier).unwrap_or_else(|_| Context::new()),
            None => Context::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.completed_at.is_some()
            && self.completed_at.unwrap() + Duration::seconds(self.ttl_duration as i64)
                < Local::now().naive_local()
    }
}

// Avro Serialization

impl AvroSerializable for Task {
    fn schema() -> &'static Schema {
        lazy_static::lazy_static! {
            static ref AVRO_SCHEMA: Schema = Schema::parse_str(
                include_str!("schemas/avro/task.json")
            ).expect("Failed to parse Avro schema");
        }
        &AVRO_SCHEMA
    }
}

// Context Extraction (this was a motherfucker)

struct HashMapExtractor<'a>(&'a std::collections::HashMap<String, String>);

impl Extractor for HashMapExtractor<'_> {
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
        _ => Err(Error::ContextExtractionError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_avro_serde() {
        let task = Task::new("", "", 1, 1)
            .with_input_data(b"".to_vec())
            .with_output_data(b"".to_vec());

        // Serialize to Avro bytes
        let avro_bytes = task.into_avro_bytes();

        // Deserialize from Avro bytes
        let deserialized = Task::from_avro_bytes(&avro_bytes);

        println!("avro_bytes: {:?}", avro_bytes.len());

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.worker_kind, deserialized.worker_kind);
        assert_eq!(task.task_kind, deserialized.task_kind);
        assert_eq!(task.input_data, deserialized.input_data);
        assert_eq!(task.status, deserialized.status);
    }

    #[test]
    fn test_task_avro_serialization_benchmark() {
        let n = 10_000;
        let task = Task::new("", "", 1, 1)
            .with_input_data(b"".to_vec())
            .with_output_data(b"".to_vec());

        // Benchmark serialization
        let start = std::time::Instant::now();
        let mut serialized_results = Vec::with_capacity(n);
        for _ in 0..n {
            let result = task.into_avro_bytes();
            serialized_results.push(result);
        }
        let ser_duration = start.elapsed();

        // Benchmark deserialization
        let start = std::time::Instant::now();
        for result in serialized_results {
            let _task_deserialized = Task::from_avro_bytes(&result);
        }
        let deser_duration = start.elapsed();

        println!("Serialization stats:");
        println!("• Total time ({} iterations): {:?}", n, ser_duration);
        println!(
            "• Average time per iteration: {:?}",
            ser_duration / n as u32
        );
        println!(
            "• Iterations per second: {:.2}",
            n as f64 / ser_duration.as_secs_f64()
        );

        println!("\nDeserialization stats:");
        println!("• Total time ({} iterations): {:?}", n, deser_duration);
        println!(
            "• Average time per iteration: {:?}",
            deser_duration / n as u32
        );
        println!(
            "• Iterations per second: {:.2}",
            n as f64 / deser_duration.as_secs_f64()
        );
    }
}
