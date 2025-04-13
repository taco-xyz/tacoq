use crate::models::{serde_avro_datetime, AvroSerializable};
use apache_avro::Schema;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// TaskRunningUpdate represents an update to a task when it starts running.
///
/// # Fields
/// * `id` - The id of the task
/// * `started_at` - The timestamp when the task started running
/// * `executed_by` - The worker that executed the task
/// * `update_type` - The type of update
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRunningUpdate {
    pub id: Uuid,
    #[serde(with = "serde_avro_datetime")]
    pub started_at: NaiveDateTime,
    pub executed_by: String,
    #[serde(default = "TaskRunningUpdate::update_type")]
    pub update_type: String,
}

// ----------------------------------------------------------------------------
// Constructors
// ----------------------------------------------------------------------------

impl TaskRunningUpdate {
    fn update_type() -> String {
        "Running".to_string()
    }

    pub fn validate_update_type(&self) -> Result<(), String> {
        if self.update_type != "Running" {
            return Err(format!(
                "Invalid update type. Expected 'Running', got '{}'",
                self.update_type
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
impl TaskRunningUpdate {
    /// Creates a new TaskRunningUpdate with the specified parameters.
    ///
    /// # Arguments
    /// * `id` - The id of the task
    /// * `started_at` - The timestamp when the task started running
    /// * `executed_by` - The worker that executed the task
    ///
    /// # Returns
    /// A new TaskRunningUpdate instance
    pub fn new(id: Uuid, started_at: NaiveDateTime, executed_by: String) -> Self {
        Self {
            id,
            started_at,
            executed_by,
            update_type: Self::update_type(),
        }
    }

    /// Creates a new TaskRunningUpdate with just the id.
    ///
    /// # Arguments
    /// * `id` - The id of the task
    ///
    /// # Returns
    /// A new TaskRunningUpdate instance
    pub fn _with_id(id: Uuid) -> Self {
        Self {
            id,
            started_at: NaiveDateTime::MIN,
            executed_by: String::new(),
            update_type: Self::update_type(),
        }
    }

    /// Sets the started_at timestamp.
    ///
    /// # Arguments
    /// * `started_at` - The timestamp when the task started running
    ///
    /// # Returns
    /// A new TaskRunningUpdate instance
    pub fn _with_started_at(mut self, started_at: NaiveDateTime) -> Self {
        self.started_at = started_at;
        self
    }

    /// Sets the executed_by field.
    ///
    /// # Arguments
    /// * `executed_by` - The worker that executed the task
    ///
    /// # Returns
    /// A new TaskRunningUpdate instance
    pub fn _with_executed_by(mut self, executed_by: String) -> Self {
        self.executed_by = executed_by;
        self
    }
}

// ----------------------------------------------------------------------------
// Avro Serialization
// ----------------------------------------------------------------------------

impl AvroSerializable for TaskRunningUpdate {
    fn schema() -> &'static Schema {
        lazy_static::lazy_static! {
            static ref AVRO_SCHEMA: Schema = Schema::parse_str(
                include_str!("schemas/avro/task_running_update.json")
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
    fn test_task_running_update_avro_serde() {
        let mut update =
            TaskRunningUpdate::new(Uuid::new_v4(), Local::now().naive_local(), "".to_string());
        update.update_type = "Running".to_string();

        // Serialize to Avro bytes
        let avro_bytes = update.try_into_avro_bytes().unwrap();

        println!("avro_bytes: {:?}", avro_bytes.len());

        // Deserialize from Avro bytes
        let deserialized = TaskRunningUpdate::try_from_avro_bytes(&avro_bytes).unwrap();

        assert_eq!(update.id, deserialized.id);
        assert_eq!(
            update.started_at.and_utc().timestamp_micros(),
            deserialized.started_at.and_utc().timestamp_micros()
        );
        assert_eq!(update.executed_by, deserialized.executed_by);
        assert_eq!(update.update_type, deserialized.update_type);
    }

    #[test]
    fn test_task_running_validate_update_type() {
        let mut update =
            TaskRunningUpdate::new(Uuid::new_v4(), Local::now().naive_local(), "".to_string());
        update.update_type = "Running".to_string();
        assert!(update.validate_update_type().is_ok());

        update.update_type = "Wrong".to_string();
        assert!(update.validate_update_type().is_err());
    }
}
