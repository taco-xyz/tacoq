use std::time::Duration;
use tokio::time;
use tracing::{error, info, info_span, Instrument};

use crate::repo::{PgTaskRepository, TaskRepository};

#[derive(Debug, Clone)]
pub struct TaskCleanupJob {
    task_repository: PgTaskRepository,
    interval: Duration,
}

impl TaskCleanupJob {
    pub fn new(task_repository: PgTaskRepository, interval_seconds: u64) -> Self {
        Self {
            task_repository,
            interval: Duration::from_secs(interval_seconds),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Starting task cleanup job with interval of {} seconds",
            self.interval.as_secs()
        );

        let mut interval = time::interval(self.interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.clean_expired_tasks().await {
                error!("Error cleaning expired tasks: {:?}", e);
            }
        }
    }

    async fn clean_expired_tasks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let span = info_span!("clean_expired_tasks");

        async {
            info!("Running cleanup of expired tasks");

            match self.task_repository.delete_expired_tasks().await {
                Ok(count) => {
                    info!("Successfully cleaned up {} expired tasks", count);
                }
                Err(e) => {
                    error!("Failed to clean up expired tasks: {}", e);
                }
            }
        }
        .instrument(span)
        .await;

        Ok(())
    }
}
