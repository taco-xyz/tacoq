use async_trait::async_trait;

use crate::task_event_consumer::event_parsing::Event;
use crate::task_event_consumer::handler::TaskEventHandler;
use std::error::Error;
use std::sync::Arc;

#[async_trait]
pub trait TaskEventCore: Send + Sync {
    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// A Task Event Consumer consumes task events from the broker and continuously
/// updates the datastore with the latest task statuses.
///
/// NOTE: The broker implementation is not separate from the parsing or implementation
/// because we use each broker's features very specifically.
#[async_trait]
pub trait TaskEventConsumer: Send + Sync {
    /// Get the event handler for this consumer
    fn event_handler(&self) -> &TaskEventHandler;

    /// Consume messages from the broker
    async fn lifecycle(&self) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Shuts down the consumer
    fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Handle a batch of events using the consumer's event handler
    async fn handle_events(&self, events: Vec<Event>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.event_handler().handle_batch_events(events).await
    }

    // Method that returns a dyn TaskEventCore trait object that allows the API to check the health of the consumer
    async fn core(&self) -> Result<Arc<dyn TaskEventCore>, Box<dyn Error + Send + Sync>>;
}
