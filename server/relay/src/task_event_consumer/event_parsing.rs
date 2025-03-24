use crate::models::{
    AvroSerializable, TaskAssignmentUpdate, TaskCompletedUpdate, TaskRunningUpdate,
};
use std::{clone::Clone, fmt::Debug};
use tracing::error;

/// Errors that can occur when processing a message.
#[derive(Debug, thiserror::Error)]
pub enum MessageProcessingError {
    #[error("Error deserializing Avro message: {0}")]
    AvroDeserializationError(String),
    #[error("Unknown message type: {0}")]
    UnknownMessageType(String),
}

/// The type of the task event. This is used to distinguish between events
/// and determine how to parse the message body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Assignment,
    Completed,
    Running,
}

impl TryFrom<String> for EventType {
    type Error = MessageProcessingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "TaskAssignment" => Ok(EventType::Assignment),
            "TaskCompleted" => Ok(EventType::Completed),
            "TaskRunning" => Ok(EventType::Running),
            _ => Err(MessageProcessingError::UnknownMessageType(value)),
        }
    }
}

impl From<EventType> for &str {
    fn from(value: EventType) -> Self {
        match value {
            EventType::Assignment => "TaskAssignment",
            EventType::Completed => "TaskCompleted",
            EventType::Running => "TaskRunning",
        }
    }
}

/// The type of the message that comes in the header.
#[derive(Debug)]
pub enum Event {
    Assignment(TaskAssignmentUpdate),
    Completed(TaskCompletedUpdate),
    Running(TaskRunningUpdate),
}

/// Based on the event type, parses raw bytes into an Event with the decoded
/// data inside.
///
/// # Arguments
///
/// * `event_type` - The type of the event to parse.
/// * `raw_bytes` - The raw bytes to parse.
///
/// # Returns
///
/// An Event with the decoded data inside.
pub fn try_parse_event_from_avro_bytes(
    event_type: EventType,
    raw_bytes: &[u8],
) -> Result<Event, MessageProcessingError> {
    match event_type {
        EventType::Assignment => {
            let assignment: TaskAssignmentUpdate =
                TaskAssignmentUpdate::try_from_avro_bytes(raw_bytes)
                    .map_err(|e| MessageProcessingError::AvroDeserializationError(e.to_string()))?;
            Ok(Event::Assignment(assignment))
        }
        EventType::Completed => {
            let completed: TaskCompletedUpdate =
                TaskCompletedUpdate::try_from_avro_bytes(raw_bytes)
                    .map_err(|e| MessageProcessingError::AvroDeserializationError(e.to_string()))?;
            Ok(Event::Completed(completed))
        }
        EventType::Running => {
            let running: TaskRunningUpdate = TaskRunningUpdate::try_from_avro_bytes(raw_bytes)
                .map_err(|e| MessageProcessingError::AvroDeserializationError(e.to_string()))?;
            Ok(Event::Running(running))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskAssignmentUpdate, TaskCompletedUpdate, TaskRunningUpdate};
    use chrono::Local;
    use uuid::Uuid;

    fn create_test_assignment() -> TaskAssignmentUpdate {
        let mut otel_ctx = std::collections::HashMap::new();
        otel_ctx.insert("trace_id".to_string(), "123".to_string());
        otel_ctx.insert("span_id".to_string(), "456".to_string());

        TaskAssignmentUpdate {
            id: Uuid::new_v4(),
            task_kind: "test_task".to_string(),
            worker_kind: "test_worker".to_string(),
            created_at: Local::now().naive_local(),
            input_data: Some(vec![1, 2, 3]),
            priority: 1,
            ttl_duration: 3600000000, // 1 hour in microseconds
            otel_ctx_carrier: otel_ctx,
        }
    }

    fn create_test_completed() -> TaskCompletedUpdate {
        TaskCompletedUpdate {
            id: Uuid::new_v4(),
            completed_at: Local::now().naive_local(),
            output_data: Some(vec![4, 5, 6]),
            is_error: 0,
        }
    }

    fn create_test_running() -> TaskRunningUpdate {
        TaskRunningUpdate {
            id: Uuid::new_v4(),
            started_at: Local::now().naive_local(),
            executed_by: "test_worker".to_string(),
        }
    }

    #[test]
    fn test_parse_assignment_event() {
        let assignment = create_test_assignment();
        let avro_bytes = assignment.try_into_avro_bytes().unwrap();

        let event = try_parse_event_from_avro_bytes(EventType::Assignment, &avro_bytes).unwrap();
        match event {
            Event::Assignment(parsed) => {
                assert_eq!(assignment.id, parsed.id);
                assert_eq!(assignment.task_kind, parsed.task_kind);
                assert_eq!(assignment.worker_kind, parsed.worker_kind);
                assert_eq!(
                    assignment.created_at.and_utc().timestamp_micros(),
                    parsed.created_at.and_utc().timestamp_micros()
                );
                assert_eq!(assignment.input_data, parsed.input_data);
                assert_eq!(assignment.priority, parsed.priority);
                assert_eq!(assignment.ttl_duration, parsed.ttl_duration);
                assert_eq!(assignment.otel_ctx_carrier, parsed.otel_ctx_carrier);
            }
            _ => panic!("Expected Assignment event"),
        }
    }

    #[test]
    fn test_parse_completed_event() {
        let completed = create_test_completed();
        let avro_bytes = completed.try_into_avro_bytes().unwrap();

        let event = try_parse_event_from_avro_bytes(EventType::Completed, &avro_bytes).unwrap();
        match event {
            Event::Completed(parsed) => {
                assert_eq!(completed.id, parsed.id);
                assert_eq!(
                    completed.completed_at.and_utc().timestamp_micros(),
                    parsed.completed_at.and_utc().timestamp_micros()
                );
                assert_eq!(completed.output_data, parsed.output_data);
                assert_eq!(completed.is_error, parsed.is_error);
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[test]
    fn test_parse_running_event() {
        let running = create_test_running();
        let avro_bytes = running.try_into_avro_bytes().unwrap();

        let event = try_parse_event_from_avro_bytes(EventType::Running, &avro_bytes).unwrap();
        match event {
            Event::Running(parsed) => {
                assert_eq!(running.id, parsed.id);
                assert_eq!(
                    running.started_at.and_utc().timestamp_micros(),
                    parsed.started_at.and_utc().timestamp_micros()
                );
                assert_eq!(running.executed_by, parsed.executed_by);
            }
            _ => panic!("Expected Running event"),
        }
    }

    #[test]
    fn test_parse_invalid_event_type() {
        let assignment = create_test_assignment();
        let avro_bytes = assignment.try_into_avro_bytes().unwrap();

        // Try to parse assignment bytes as completed event
        let result = try_parse_event_from_avro_bytes(EventType::Completed, &avro_bytes);
        assert!(result.is_err());

        // Try to parse assignment bytes as running event
        let result = try_parse_event_from_avro_bytes(EventType::Running, &avro_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_bytes() {
        let invalid_bytes = vec![0, 1, 2, 3]; // Invalid Avro bytes

        let result = try_parse_event_from_avro_bytes(EventType::Assignment, &invalid_bytes);
        assert!(result.is_err());

        let result = try_parse_event_from_avro_bytes(EventType::Completed, &invalid_bytes);
        assert!(result.is_err());

        let result = try_parse_event_from_avro_bytes(EventType::Running, &invalid_bytes);
        assert!(result.is_err());
    }
}
