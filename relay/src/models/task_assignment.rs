use crate::models::{serde_avro_datetime, AvroSerializable};
use apache_avro::{serde_avro_bytes, Schema};
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
/// * `update_type` - The type of update - IMPORTANT NOTE: This isn't useless.
///   if the task happens to be deserialized from a message with the same byte
///   count, an error won't be thrown but the data will be totally f-ed. We
///   ALWAYS need to validate that the update_type is correct and matches the
///   expected type.
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskAssignmentUpdate {
    pub id: Uuid,
    pub task_kind: String,
    pub worker_kind: String,
    #[serde(with = "serde_avro_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serde_avro_bytes")]
    pub input_data: Vec<u8>,
    pub priority: i32,
    pub ttl_duration: i64,
    pub otel_ctx_carrier: std::collections::HashMap<String, String>,
    #[serde(default = "TaskAssignmentUpdate::update_type")]
    pub update_type: String,
}

impl TaskAssignmentUpdate {
    fn update_type() -> String {
        "Assignment".to_string()
    }

    pub fn validate_update_type(&self) -> Result<(), String> {
        if self.update_type != "Assignment" {
            return Err(format!(
                "Invalid update type. Expected 'Assignment', got '{}'",
                self.update_type
            ));
        }
        Ok(())
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
    use std::collections::HashMap;

    use super::*;
    use chrono::Local;

    #[test]
    fn test_task_assignment_avro_serde() {
        let mut otel_ctx = std::collections::HashMap::new();
        otel_ctx.insert("trace_id".to_string(), "123".to_string());
        otel_ctx.insert("span_id".to_string(), "456".to_string());

        let assignment = TaskAssignmentUpdate {
            id: Uuid::new_v4(),
            task_kind: "test_task".to_string(),
            worker_kind: "test_worker".to_string(),
            created_at: Local::now().naive_local(),
            input_data: vec![1, 2, 3],
            priority: 1,
            ttl_duration: 3600000000, // 1 hour in microseconds
            otel_ctx_carrier: otel_ctx.clone(),
            update_type: "Assignment".to_string(),
        };

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

    #[test]
    fn test_task_assignment_validate_update_type() {
        let mut assignment = TaskAssignmentUpdate {
            id: Uuid::new_v4(),
            task_kind: "test_task".to_string(),
            worker_kind: "test_worker".to_string(),
            created_at: Local::now().naive_local(),
            input_data: vec![1, 2, 3],
            priority: 1,
            ttl_duration: 3600000000,
            otel_ctx_carrier: HashMap::new(),
            update_type: "Assignment".to_string(),
        };

        assert!(assignment.validate_update_type().is_ok());

        assignment.update_type = "Wrong".to_string();
        assert!(assignment.validate_update_type().is_err());
    }
}
