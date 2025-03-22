// Standard/External imports
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

// Avro-specific imports
use apache_avro::{
    from_avro_datum, from_value, serde_avro_bytes_opt, to_avro_datum, types::Value, Schema,
};

/// Tasks are sent to workers to be executed with a specific payload.
/// Workers are eligble for receiving certain tasks depending on their
/// list of capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAvro {
    pub id: Uuid,
    pub task_kind: String,

    #[serde(with = "serde_avro_bytes_opt")]
    pub input_data: Option<Vec<u8>>,
    #[serde(with = "serde_avro_bytes_opt")]
    pub output_data: Option<Vec<u8>>,
    pub is_error: i32,

    pub status: String,
    pub priority: i32,

    // Relations
    pub worker_kind: String,
    pub executed_by: Option<String>, // worker that it is assigned to

    // Task status
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,

    pub ttl_duration: i64, // in seconds

    // Metadata
    pub created_at: i64,
    pub updated_at: i64,

    // OpenTelemetry context carrier
    pub otel_ctx_carrier: Option<JsonValue>,
}

// Helper functions
mod helpers {
    use chrono::{DateTime, NaiveDateTime};

    pub fn timestamp_to_datetime(ts: i64) -> Option<NaiveDateTime> {
        DateTime::from_timestamp_micros(ts * 1_000_000).map(|dt| dt.naive_utc())
    }

    pub fn datetime_to_timestamp(dt: NaiveDateTime) -> i64 {
        dt.and_utc().timestamp()
    }
}

// Conversion traits in a separate module
mod conversions {
    use super::helpers;
    use crate::models::{Task, TaskAvro};

    impl From<Task> for TaskAvro {
        fn from(task: Task) -> Self {
            TaskAvro {
                id: task.id,
                task_kind: task.task_kind,
                input_data: task.input_data,
                output_data: task.output_data,
                is_error: task.is_error,
                status: task.status.to_string(),
                priority: task.priority,
                worker_kind: task.worker_kind,
                executed_by: task.executed_by,
                started_at: task.started_at.map(helpers::datetime_to_timestamp),
                completed_at: task.completed_at.map(helpers::datetime_to_timestamp),
                ttl_duration: task.ttl_duration,
                created_at: helpers::datetime_to_timestamp(task.created_at),
                updated_at: helpers::datetime_to_timestamp(task.updated_at),
                otel_ctx_carrier: task.otel_ctx_carrier,
            }
        }
    }

    impl From<TaskAvro> for Task {
        fn from(value: TaskAvro) -> Self {
            Task {
                id: value.id,
                task_kind: value.task_kind,
                input_data: value.input_data,
                output_data: value.output_data,
                is_error: value.is_error,
                status: value.status.into(),
                priority: value.priority,
                worker_kind: value.worker_kind,
                executed_by: value.executed_by,
                started_at: value.started_at.and_then(helpers::timestamp_to_datetime),
                completed_at: value.completed_at.and_then(helpers::timestamp_to_datetime),
                ttl_duration: value.ttl_duration,
                created_at: helpers::timestamp_to_datetime(value.created_at).unwrap_or_default(),
                updated_at: helpers::timestamp_to_datetime(value.updated_at).unwrap_or_default(),
                otel_ctx_carrier: value.otel_ctx_carrier,
            }
        }
    }
}

// Avro-specific implementations
impl TaskAvro {
    // Schema definition
    fn schema() -> &'static Schema {
        lazy_static::lazy_static! {
            static ref AVRO_SCHEMA: Schema = Schema::parse_str(
                include_str!("schemas/avro/task.json")
            ).expect("Failed to parse Avro schema");
        }
        &AVRO_SCHEMA
    }

    // Serialization methods
    pub fn into_avro_bytes(&self) -> Vec<u8> {
        let datum = Value::Record(vec![
            ("id".to_string(), self.id.into()),
            ("task_kind".to_string(), self.task_kind.clone().into()),
            ("input_data".to_string(), self.input_data.clone().into()),
            ("output_data".to_string(), self.output_data.clone().into()),
            ("is_error".to_string(), self.is_error.into()),
            ("status".to_string(), self.status.clone().into()),
            ("priority".to_string(), self.priority.into()),
            ("worker_kind".to_string(), self.worker_kind.clone().into()),
            ("executed_by".to_string(), self.executed_by.clone().into()),
            ("started_at".to_string(), self.started_at.clone().into()),
            ("completed_at".to_string(), self.completed_at.clone().into()),
            ("ttl_duration".to_string(), self.ttl_duration.into()),
            ("created_at".to_string(), self.created_at.into()),
            ("updated_at".to_string(), self.updated_at.into()),
            (
                "otel_ctx_carrier".to_string(),
                self.otel_ctx_carrier.clone().into(),
            ),
        ]);

        to_avro_datum(Self::schema(), datum).unwrap()
    }

    pub fn from_avro_bytes(data: &[u8]) -> Self {
        let mut reader = data;
        let value = from_avro_datum(Self::schema(), &mut reader, None).unwrap();
        from_value::<TaskAvro>(&value).unwrap()
    }
}

impl PartialEq for TaskAvro {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.task_kind == other.task_kind
            && self.input_data == other.input_data
            && self.output_data == other.output_data
            && self.is_error == other.is_error
            && self.status == other.status
            && self.priority == other.priority
            && self.worker_kind == other.worker_kind
            && self.executed_by == other.executed_by
            && self.started_at == other.started_at
            && self.completed_at == other.completed_at
            && self.ttl_duration == other.ttl_duration
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
            && self.otel_ctx_carrier == other.otel_ctx_carrier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;

    #[test]
    fn test_task_avro_serialization() {
        let task = Task::new("", "", 1, 1)
            .with_input_data(b"".to_vec())
            .with_output_data(b"".to_vec());

        let task_avro: TaskAvro = task.into();
        let result = task_avro.into_avro_bytes();
        let task_avro_deserialized = TaskAvro::from_avro_bytes(&result);
        assert_eq!(task_avro, task_avro_deserialized);
        println!("Bytes: {:?}", result.len());
    }

    #[test]
    fn test_task_avro_serialization_benchmark() {
        let n = 100_000;
        let task = Task::new("", "", 1, 1)
            .with_input_data(b"".to_vec())
            .with_output_data(b"".to_vec());

        let task_avro: TaskAvro = task.into();

        // Benchmark serialization
        let start = std::time::Instant::now();
        let mut serialized_results = Vec::with_capacity(n);
        for _ in 0..n {
            let result = task_avro.into_avro_bytes();
            serialized_results.push(result);
        }
        let ser_duration = start.elapsed();

        // Benchmark deserialization
        let start = std::time::Instant::now();
        for result in serialized_results {
            let _task_avro_deserialized = TaskAvro::from_avro_bytes(&result);
        }
        let deser_duration = start.elapsed();

        println!(
            "Serialization - Total time for {} iterations: {:?}",
            n, ser_duration
        );
        println!(
            "Serialization - Average time per iteration: {:?}",
            ser_duration / n as u32
        );
        println!(
            "Deserialization - Total time for {} iterations: {:?}",
            n, deser_duration
        );
        println!(
            "Deserialization - Average time per iteration: {:?}",
            deser_duration / n as u32
        );
    }
}
