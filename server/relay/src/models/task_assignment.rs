use crate::models::{serde_avro_datetime, AvroSerializable};
use apache_avro::{serde_avro_bytes_opt, Schema};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// TaskAssignmentUpdate represents a task being assigned to a worker.
///
/// # Fields
/// * `id` - The id of the task
/// * `task_kind` - The type of task
/// * `worker_kind` - The type of worker that can execute this task
/// * `created_at` - The timestamp when the task was created
/// * `input_data` - Optional input data for the task
/// * `priority` - The priority of the task
/// * `ttl_duration` - Time to live duration in microseconds
/// * `otel_ctx_carrier` - OpenTelemetry context carrier map
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskAssignmentUpdate {
    pub id: Uuid,
    pub task_kind: String,
    pub worker_kind: String,
    #[serde(with = "serde_avro_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serde_avro_bytes_opt")]
    pub input_data: Option<Vec<u8>>,
    pub priority: i32,
    pub ttl_duration: i64,
    pub otel_ctx_carrier: std::collections::HashMap<String, String>,
}

// ----------------------------------------------------------------------------
// Constructors
// ----------------------------------------------------------------------------

impl TaskAssignmentUpdate {
    /// Creates a new TaskAssignmentUpdate with the specified parameters.
    pub fn new(
        id: Uuid,
        task_kind: String,
        worker_kind: String,
        created_at: NaiveDateTime,
        input_data: Option<Vec<u8>>,
        priority: i32,
        ttl_duration: i64,
        otel_ctx_carrier: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            task_kind,
            worker_kind,
            created_at,
            input_data,
            priority,
            ttl_duration,
            otel_ctx_carrier,
        }
    }

    /// Creates a new TaskAssignmentUpdate with just the id.
    pub fn with_id(id: Uuid) -> Self {
        Self {
            id,
            task_kind: String::new(),
            worker_kind: String::new(),
            created_at: NaiveDateTime::MIN,
            input_data: None,
            priority: 0,
            ttl_duration: 0,
            otel_ctx_carrier: std::collections::HashMap::new(),
        }
    }

    /// Sets the task_kind field.
    pub fn with_task_kind(mut self, task_kind: String) -> Self {
        self.task_kind = task_kind;
        self
    }

    /// Sets the worker_kind field.
    pub fn with_worker_kind(mut self, worker_kind: String) -> Self {
        self.worker_kind = worker_kind;
        self
    }

    /// Sets the created_at timestamp.
    pub fn with_created_at(mut self, created_at: NaiveDateTime) -> Self {
        self.created_at = created_at;
        self
    }

    /// Sets the input_data field.
    pub fn with_input_data(mut self, input_data: Option<Vec<u8>>) -> Self {
        self.input_data = input_data;
        self
    }

    /// Sets the priority field.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the ttl_duration field.
    pub fn with_ttl_duration(mut self, ttl_duration: i64) -> Self {
        self.ttl_duration = ttl_duration;
        self
    }

    /// Sets the otel_ctx_carrier field.
    pub fn with_otel_ctx_carrier(
        mut self,
        otel_ctx_carrier: std::collections::HashMap<String, String>,
    ) -> Self {
        self.otel_ctx_carrier = otel_ctx_carrier;
        self
    }
}

// ----------------------------------------------------------------------------
// Avro Serialization
// ----------------------------------------------------------------------------

impl AvroSerializable for TaskAssignmentUpdate {
    fn schema() -> &'static Schema {
        lazy_static::lazy_static! {
            static ref AVRO_SCHEMA: Schema = Schema::parse_str(
                include_str!("schemas/avro/task_assignment_update.json")
            ).expect("Failed to parse Avro schema");
        }
        &AVRO_SCHEMA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_task_assignment_avro_serde() {
        let mut otel_ctx = std::collections::HashMap::new();
        otel_ctx.insert("trace_id".to_string(), "123".to_string());
        otel_ctx.insert("span_id".to_string(), "456".to_string());

        let assignment = TaskAssignmentUpdate::new(
            Uuid::new_v4(),
            "test_task".to_string(),
            "test_worker".to_string(),
            Local::now().naive_local(),
            Some(vec![1, 2, 3]),
            1,
            3600000000, // 1 hour in microseconds
            otel_ctx.clone(),
        );

        // Serialize to Avro bytes
        let avro_bytes = assignment.try_into_avro_bytes().unwrap();

        // Deserialize from Avro bytes
        let deserialized = TaskAssignmentUpdate::try_from_avro_bytes(&avro_bytes).unwrap();

        assert_eq!(assignment.id, deserialized.id);
        assert_eq!(assignment.task_kind, deserialized.task_kind);
        assert_eq!(assignment.worker_kind, deserialized.worker_kind);
        // Compare timestamps only up to microsecond precision
        assert_eq!(
            assignment.created_at.and_utc().timestamp_micros(),
            deserialized.created_at.and_utc().timestamp_micros()
        );
        assert_eq!(assignment.input_data, deserialized.input_data);
        assert_eq!(assignment.priority, deserialized.priority);
        assert_eq!(assignment.ttl_duration, deserialized.ttl_duration);
        assert_eq!(assignment.otel_ctx_carrier, deserialized.otel_ctx_carrier);
    }
}
