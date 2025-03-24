use std::collections::HashMap;

use chrono::{Local, NaiveDateTime};
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

// Task status enum
/// # Possible Status:
/// * `Pending`: Task is created but not yet assigned
/// * `Processing`: Task has been assigned to a worker and sent to a queue
/// * `Completed`: Task completed successfully or not
#[derive(Display, EnumString, Debug, PartialEq, ToSchema, Clone)]
pub enum TaskStatus {
    Pending,    // Task is created but not yet assigned
    Processing, // Task has been assigned to a worker and sent to a queue
    Completed,  // Task completed successfully or not
}

/// Tasks are sent to workers to be executed with a specific payload.
/// Workers are eligble for receiving certain tasks depending on their
/// list of capabilities.
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    #[sqlx(rename = "task_kind_name")]
    pub task_kind: Option<String>,

    // Task data
    #[serde(with = "serde_avro_bytes_opt")]
    pub input_data: Option<Vec<u8>>, // byte array
    #[serde(with = "serde_avro_bytes_opt")]
    pub output_data: Option<Vec<u8>>, // byte array
    pub is_error: Option<i32>,

    pub priority: Option<i32>,

    // Relations
    #[sqlx(rename = "worker_kind_name")]
    pub worker_kind: Option<String>,
    pub executed_by: Option<String>, // worker that it is assigned to

    // Task status
    #[serde(with = "serde_avro_datetime_opt")]
    pub started_at: Option<NaiveDateTime>,
    #[serde(with = "serde_avro_datetime_opt")]
    pub completed_at: Option<NaiveDateTime>,

    pub ttl_duration: Option<i64>, // in seconds

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
            task_kind: Some(task_kind_name.to_string()),
            input_data: None,
            output_data: None,
            is_error: Some(0),
            priority: Some(priority),
            worker_kind: Some(worker_kind_name.to_string()),
            executed_by: None,
            started_at: None,
            completed_at: None,
            ttl_duration: Some(ttl_duration),
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
        self.is_error = if is_error { Some(1) } else { Some(0) };
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

    /// Returns the status of the task.
    pub fn status(&self) -> TaskStatus {
        if self.completed_at.is_some() {
            TaskStatus::Completed
        } else if self.started_at.is_some() {
            TaskStatus::Processing
        } else {
            TaskStatus::Pending
        }
    }

    /// Returns the context of the task.
    pub fn context(&self) -> Context {
        let carrier_value = self.otel_ctx_carrier.clone();
        match carrier_value {
            Some(carrier) => extract_context(&carrier).unwrap_or_else(|_| Context::new()),
            None => Context::new(),
        }
    }
}

// ----------------------------------------------------------------------------
// Avro Serialization
// ----------------------------------------------------------------------------

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

// ----------------------------------------------------------------------------
// Context Extraction (this was a motherfucker)
// ----------------------------------------------------------------------------

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

/// Extracts the context from the carrier.
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
    use serde_json::json;

    use super::*;

    #[test]
    fn test_task_avro_serde() {
        let mut task = Task::new(
            "delayed_instrumented_task",
            "70ab4a91-04c4-4670-91e1-cb81153d4e0f",
            0,
            604800,
        );
        task.id = Uuid::parse_str("6ff84e6d-f40d-4617-9874-ce625d59a0d5").unwrap();
        task.executed_by = Some("70ab4a91-04c4-4670-91e1-cb81153d4e0f".to_string());
        task.started_at = Some(
            NaiveDateTime::parse_from_str("2025-03-23T05:03:59.995089", "%Y-%m-%dT%H:%M:%S.%f")
                .unwrap(),
        );
        task.input_data = Some(b"{\"test\": \"data\"}".to_vec());
        task.otel_ctx_carrier = Some(json!({
            "traceparent": "00-3f4a168998a20c019615e558ec12d985-47d2075b594ffe86-01"
        }));

        // Serialize to Avro bytes
        let avro_bytes = task.try_into_avro_bytes().unwrap();

        // Deserialize from Avro bytes
        let deserialized = Task::try_from_avro_bytes(&avro_bytes).unwrap();

        println!("avro_bytes: {:?}", avro_bytes.len());

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.worker_kind, deserialized.worker_kind);
        assert_eq!(task.task_kind, deserialized.task_kind);
        assert_eq!(task.input_data, deserialized.input_data);
    }
}
