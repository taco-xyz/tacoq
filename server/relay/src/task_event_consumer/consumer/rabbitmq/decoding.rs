use crate::task_event_consumer::event_parsing::{
    try_parse_event_from_avro_bytes, Event, EventType, MessageProcessingError,
};
use lapin::message::Delivery;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodingError {
    #[error("No headers found in message")]
    NoHeadersFound,
    #[error("No message_type found in headers")]
    NoMessageTypeFound,
    #[error("Invalid message_type format")]
    InvalidMessageTypeFormat,
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
    #[error("Failed to parse event: {0}")]
    EventParsingError(#[from] MessageProcessingError),
}

/// Decodes a RabbitMQ delivery into an Event based on the message type in
/// headers.
fn decode_delivery(delivery: &Delivery) -> Result<Event, DecodingError> {
    let headers = delivery
        .properties
        .headers()
        .as_ref()
        .ok_or(DecodingError::NoHeadersFound)?;

    let message_type = headers
        .inner()
        .get("message_type")
        .ok_or(DecodingError::NoMessageTypeFound)?
        .as_long_string()
        .ok_or(DecodingError::InvalidMessageTypeFormat)?
        .to_string();

    let event_type: EventType = EventType::try_from(message_type)
        .map_err(|e| DecodingError::InvalidMessageType(e.to_string()))?;

    try_parse_event_from_avro_bytes(event_type, &delivery.data)
        .map_err(DecodingError::EventParsingError)
}

impl TryFrom<Delivery> for Event {
    type Error = DecodingError;

    fn try_from(delivery: Delivery) -> Result<Self, Self::Error> {
        decode_delivery(&delivery)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AvroSerializable, TaskAssignmentUpdate, TaskCompletedUpdate, TaskRunningUpdate,
    };
    use chrono::Local;
    use lapin::acker::Acker;
    use lapin::message::Delivery;
    use lapin::types::{AMQPValue, FieldTable};
    use lapin::BasicProperties;
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
            ttl_duration: 3600000000,
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

    fn create_headers(message_type: &str) -> FieldTable {
        let mut headers = FieldTable::default();
        headers.insert(
            "message_type".to_string().into(),
            AMQPValue::LongString(message_type.to_string().into()),
        );
        headers
    }

    fn create_delivery(data: Vec<u8>, headers: FieldTable) -> Delivery {
        Delivery {
            delivery_tag: 0,
            exchange: "".to_string().into(),
            routing_key: "".to_string().into(),
            data,
            redelivered: false,
            properties: BasicProperties::default().with_headers(headers),
            acker: Acker::default(),
        }
    }

    #[test]
    fn test_decode_assignment_event() {
        let assignment = create_test_assignment();
        let avro_bytes = assignment.try_into_avro_bytes().unwrap();
        let delivery = create_delivery(avro_bytes, create_headers(EventType::Assignment.into()));

        let event = Event::try_from(delivery).unwrap();
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
    fn test_decode_completed_event() {
        let completed = create_test_completed();
        let avro_bytes = completed.try_into_avro_bytes().unwrap();
        let delivery = create_delivery(avro_bytes, create_headers(EventType::Completed.into()));
        let event = Event::try_from(delivery).unwrap();
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
    fn test_decode_running_event() {
        let running = create_test_running();
        let avro_bytes = running.try_into_avro_bytes().unwrap();
        let delivery = create_delivery(avro_bytes, create_headers(EventType::Running.into()));

        let event = Event::try_from(delivery).unwrap();
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
    fn test_decode_missing_headers() {
        let delivery = Delivery {
            delivery_tag: 0,
            exchange: "".to_string().into(),
            routing_key: "".to_string().into(),
            data: vec![],
            redelivered: false,
            properties: BasicProperties::default(),
            acker: Acker::default(),
        };

        let result = Event::try_from(delivery);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecodingError::NoHeadersFound));
    }

    #[test]
    fn test_decode_missing_message_type() {
        let mut headers = FieldTable::default();
        headers.insert(
            "other_header".to_string().into(),
            AMQPValue::LongString("value".to_string().into()),
        );

        let delivery = Delivery {
            delivery_tag: 0,
            exchange: "".to_string().into(),
            routing_key: "".to_string().into(),
            data: vec![],
            redelivered: false,
            properties: BasicProperties::default().with_headers(headers),
            acker: Acker::default(),
        };

        let result = Event::try_from(delivery);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DecodingError::NoMessageTypeFound
        ));
    }

    #[test]
    fn test_decode_invalid_message_type() {
        let delivery = create_delivery(vec![], create_headers("InvalidType"));

        let result = Event::try_from(delivery);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DecodingError::InvalidMessageType(_)
        ));
    }

    #[test]
    fn test_decode_invalid_avro_data() {
        let delivery = create_delivery(
            vec![0, 1, 2, 3],
            create_headers(EventType::Assignment.into()),
        );
        let result = Event::try_from(delivery);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DecodingError::EventParsingError(_)
        ));
    }

    #[test]
    fn test_decode_mismatched_event_type() {
        let assignment = create_test_assignment();
        let avro_bytes = assignment.try_into_avro_bytes().unwrap();
        let delivery = create_delivery(avro_bytes, create_headers(EventType::Completed.into()));

        let result = Event::try_from(delivery);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DecodingError::EventParsingError(_)
        ));
    }
}
