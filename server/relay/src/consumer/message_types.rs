use crate::models::{TaskAssignmentUpdate, TaskCompletedUpdate, TaskRunningUpdate};
use std::{clone::Clone, fmt::Debug};
use tracing::error;

/// The type of the message that comes in the header.
#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Assignment,
    Completed,
    Running,
}

/// Errors that can occur when processing a message.
#[derive(Debug, thiserror::Error)]
pub enum MessageProcessingError {
    #[error("Unknown message type: {0}")]
    UnknownMessageType(String),
}

impl TryFrom<String> for MessageType {
    type Error = MessageProcessingError;

    fn try_from(message_type: String) -> Result<Self, Self::Error> {
        match message_type.as_str() {
            "TaskAssignment" => Ok(MessageType::Assignment),
            "TaskCompleted" => Ok(MessageType::Completed),
            "TaskRunning" => Ok(MessageType::Running),
            _ => Err(MessageProcessingError::UnknownMessageType(message_type)),
        }
    }
}

/// The type of the message that comes in the header.
#[derive(Debug)]
pub enum Message {
    Assignment(TaskAssignmentUpdate),
    Completed(TaskCompletedUpdate),
    Running(TaskRunningUpdate),
}
