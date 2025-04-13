use crate::repo::task_repo::TaskRepository;
use crate::task_event_consumer::event_parsing::Event;
use std::error::Error;
use std::sync::Arc;

/// A Task Event Handler handles task events in the consumer.
///
/// It is separate from the consumer definition because a Redis consumer
/// implementation would use the same handler as a RabbitMQ consumer.
///
/// It is NOT a trait because we know we will always be using the repositories
/// for uploading the events, and if we were to support more databases, they
/// would have different repositories, but the handler would remain the same.
pub struct TaskEventHandler {
    task_repository: Arc<TaskRepository>,
}

impl TaskEventHandler {
    pub fn new(task_repository: Arc<TaskRepository>) -> Self {
        Self { task_repository }
    }

    /// Uploads all the received events to the repository.
    pub async fn handle_batch_events(
        &self,
        events: Vec<Event>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // TODO - This implementation is dogshit and should only be used for
        // 1-event-at-a-time ingestion. If we want to do batches, we should batch them
        // in a Postgres transaction and upload them all at once, which requires adding
        // a new method to the TaskRepository that accepts a Vec<Update>
        for event in events {
            match event {
                Event::Assignment(assignment) => {
                    self.task_repository
                        .update_task_from_assignment_update(&assignment)
                        .await?;
                }
                Event::Completed(completed) => {
                    self.task_repository
                        .update_task_from_completed_update(&completed)
                        .await?;
                }
                Event::Running(running) => {
                    self.task_repository
                        .update_task_from_running_update(&running)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
