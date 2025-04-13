use crate::models::{serde_avro_datetime, AvroSerializable};
use apache_avro::{serde_avro_bytes, Schema};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// TaskCompletedUpdate represents an update to a task when it completes.
///
/// # Fields
/// * `id` - The id of the task
/// * `completed_at` - The timestamp when the task completed
/// * `output_data` - Optional output data from the task execution
/// * `is_error` - Whether the task completed with an error
/// * `update_type` - The type of update
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskCompletedUpdate {
    pub id: Uuid,
    #[serde(with = "serde_avro_datetime")]
    pub completed_at: NaiveDateTime,
    #[serde(with = "serde_avro_bytes")]
    pub output_data: Vec<u8>,
    pub is_error: i32,
    #[serde(default = "TaskCompletedUpdate::update_type")]
    pub update_type: String,
}

// ----------------------------------------------------------------------------
// Constructors
// ----------------------------------------------------------------------------
impl TaskCompletedUpdate {
    fn update_type() -> String {
        "Completed".to_string()
    }

    pub fn validate_update_type(&self) -> Result<(), String> {
        if self.update_type != "Completed" {
            return Err(format!(
                "Invalid update type. Expected 'Completed', got '{}'",
                self.update_type
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
impl TaskCompletedUpdate {
    /// Creates a new TaskCompletedUpdate with the specified parameters.
    ///
    /// # Arguments
    /// * `id` - The id of the task
    /// * `completed_at` - The timestamp when the task completed
    /// * `output_data` - Optional output data from the task execution
    /// * `is_error` - Whether the task completed with an error
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn new(id: Uuid, completed_at: NaiveDateTime, output_data: Vec<u8>, is_error: i32) -> Self {
        Self {
            id,
            completed_at,
            output_data,
            is_error,
            update_type: Self::update_type(),
        }
    }

    /// Creates a new TaskCompletedUpdate with just the id.
    ///
    /// # Arguments
    /// * `id` - The id of the task
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn _with_id(id: Uuid) -> Self {
        Self {
            id,
            completed_at: NaiveDateTime::MIN,
            output_data: vec![],
            is_error: 0,
            update_type: Self::update_type(),
        }
    }

    /// Sets the completed_at timestamp.
    ///
    /// # Arguments
    /// * `completed_at` - The timestamp when the task completed
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn _with_completed_at(mut self, completed_at: NaiveDateTime) -> Self {
        self.completed_at = completed_at;
        self
    }

    /// Sets the output_data field.
    ///
    /// # Arguments
    /// * `output_data` - Optional output data from the task execution
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn _with_output_data(mut self, output_data: Vec<u8>) -> Self {
        self.output_data = output_data;
        self
    }

    /// Sets the is_error field.
    ///
    /// # Arguments
    /// * `is_error` - Whether the task completed with an error
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn _with_is_error(mut self, is_error: i32) -> Self {
        self.is_error = is_error;
        self
    }
}

// ----------------------------------------------------------------------------
// Avro Serialization
// ----------------------------------------------------------------------------

impl AvroSerializable for TaskCompletedUpdate {
    fn schema() -> &'static Schema {
        lazy_static::lazy_static! {
            static ref AVRO_SCHEMA: Schema = Schema::parse_str(
                include_str!("schemas/avro/task_completed_update.json")
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
    fn test_task_completed_update_avro_serde() {
        let mut update =
            TaskCompletedUpdate::new(Uuid::new_v4(), Local::now().naive_local(), vec![1, 2, 3], 0);
        update.update_type = "Completed".to_string();

        // Serialize to Avro bytes
        let avro_bytes = update.try_into_avro_bytes().unwrap();

        println!("avro_bytes: {:?}", avro_bytes.len());

        // Deserialize from Avro bytes
        let deserialized = TaskCompletedUpdate::try_from_avro_bytes(&avro_bytes).unwrap();

        assert_eq!(update.id, deserialized.id);
        assert_eq!(
            update.completed_at.and_utc().timestamp_micros(),
            deserialized.completed_at.and_utc().timestamp_micros()
        );
        assert_eq!(update.output_data, deserialized.output_data);
        assert_eq!(update.is_error, deserialized.is_error);
        assert_eq!(update.update_type, deserialized.update_type);
    }

    #[test]
    fn test_task_completed_validate_update_type() {
        let mut update =
            TaskCompletedUpdate::new(Uuid::new_v4(), Local::now().naive_local(), vec![], 0);
        update.update_type = "Completed".to_string();
        assert!(update.validate_update_type().is_ok());

        update.update_type = "Wrong".to_string();
        assert!(update.validate_update_type().is_err());
    }
}
