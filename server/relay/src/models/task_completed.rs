use crate::models::{serde_avro_datetime, AvroSerializable};
use apache_avro::{serde_avro_bytes_opt, Schema};
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
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskCompletedUpdate {
    pub id: Uuid,
    #[serde(with = "serde_avro_datetime")]
    pub completed_at: NaiveDateTime,
    #[serde(with = "serde_avro_bytes_opt")]
    pub output_data: Option<Vec<u8>>,
    pub is_error: i32,
}

// ----------------------------------------------------------------------------
// Constructors
// ----------------------------------------------------------------------------

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
    pub fn new(
        id: Uuid,
        completed_at: NaiveDateTime,
        output_data: Option<Vec<u8>>,
        is_error: i32,
    ) -> Self {
        Self {
            id,
            completed_at,
            output_data,
            is_error,
        }
    }

    /// Creates a new TaskCompletedUpdate with just the id.
    ///
    /// # Arguments
    /// * `id` - The id of the task
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn with_id(id: Uuid) -> Self {
        Self {
            id,
            completed_at: NaiveDateTime::MIN,
            output_data: None,
            is_error: 0,
        }
    }

    /// Sets the completed_at timestamp.
    ///
    /// # Arguments
    /// * `completed_at` - The timestamp when the task completed
    ///
    /// # Returns
    /// A new TaskCompletedUpdate instance
    pub fn with_completed_at(mut self, completed_at: NaiveDateTime) -> Self {
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
    pub fn with_output_data(mut self, output_data: Option<Vec<u8>>) -> Self {
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
    pub fn with_is_error(mut self, is_error: i32) -> Self {
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
        let update = TaskCompletedUpdate::new(
            Uuid::new_v4(),
            Local::now().naive_local(),
            Some(vec![1, 2, 3]),
            0,
        );

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
    }
}
