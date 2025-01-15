use crate::constants;
use crate::repo::PgTaskInstanceRepository;
use common::brokers::core::MessageHandler;
use common::brokers::Broker;

use std::sync::Arc;

#[derive(Clone, Debug)]
struct Handler {
    publisher: Broker, // shared amongst controllers
    _task_repository: Arc<PgTaskInstanceRepository>, // Here maybe we should have a service to share logic publisher + task_repository
                                                     // TODO: check for other relevant repositories
}

impl Handler {
    pub async fn new(
        publisher: Broker,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        publisher.setup().await?;

        Ok(Self {
            publisher,
            _task_repository: task_repository,
        })
    }

    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.publisher.cleanup().await?;
        Ok(())
    }
}

impl MessageHandler for Handler {
    fn handle(&self, message: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        // Currently just print the message to stdout
        // Here is where the message logic for receiving messages should live
        println!("{:?}", message);

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TaskInputController {
    consumer: Broker,
    handler: Handler,
}

impl TaskInputController {
    pub async fn new(
        broker_url: &str,
        publisher: Broker,
        task_repository: Arc<PgTaskInstanceRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let s: &str = constants::TASK_INPUT_QUEUE;

        // Need to create here a new broker connection -> check if it is the appropriate place
        // TODO: change the hardcoded values into non hardcoded ones
        let consumer = Broker::new(broker_url, s, None, Some(s.to_string())).await?;
        let handler = Handler::new(publisher, task_repository).await?;

        Ok(Self { consumer, handler })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.setup().await?;
        self.consumer
            .consume(Box::new(self.handler.clone()))
            .await?;

        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Cleanup both consumer and publisher
        self.consumer.cleanup().await?;
        self.handler.cleanup().await?;
        Ok(())
    }
}
