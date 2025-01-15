use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::MessageHandler;
use common::brokers::Broker;

use std::sync::Arc;

// This file will create a rabbitmq consumer and a shared publisher with the application. This will read from the queue and publish on the rabbit queue

#[derive(Clone, Debug)]
struct TaskResultController {
    consumer: Broker,       // This should be a reader broker class
    publisher: Arc<Broker>, // shared amongst controllers
    task_repository: Arc<PgTaskInstanceRepository>, // Here maybe we should have a service to share logic publisher + task_repository
                                                    // TODO: check for other relevant repositories
}

struct Handler;
impl MessageHandler for Handler {
    fn handle(&self, message: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        // Currently just print the message to stdout
        // Here is where the message logic for receiving messages should live
        println!(message);

        Ok(())
    }
}

impl TaskResultController {
    pub async fn new(
        broker_url: &str,
        publisher: Arc<Broker>,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Need to create here a new broker connection -> check if it is the appropriate place
        // TODO: change the hardcoded values into non hardcoded ones
        let consumer = Broker::new(broker_url, "task_result", None, "task_result").await?;

        Self {
            consumer,
            publisher,
            task_repository,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let handler = Handler::new();

        self.consumer.consume(handler).await?;

        Ok(())
    }
}
