use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, info_span, warn, Instrument};

use crate::repo::TaskRepository;

#[derive(Debug, Clone)]
pub struct TaskCleanupJob {
    task_repository: TaskRepository,
    interval: Duration,
}

impl TaskCleanupJob {
    pub fn new(task_repository: TaskRepository, interval_seconds: u64) -> Self {
        info!(
            interval_seconds = interval_seconds,
            "Creating task cleanup job"
        );
        Self {
            task_repository,
            interval: Duration::from_secs(interval_seconds),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            interval_seconds = self.interval.as_secs(),
            "Starting task cleanup job"
        );

        let mut interval = time::interval(self.interval);
        let mut consecutive_errors = 0;

        loop {
            interval.tick().await;
            debug!("Task cleanup tick triggered");

            match self.clean_expired_tasks().await {
                Ok(_) => {
                    if consecutive_errors > 0 {
                        info!(
                            previous_errors = consecutive_errors,
                            "Task cleanup recovered after errors"
                        );
                        consecutive_errors = 0;
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    error!(
                        error = %e,
                        consecutive_errors = consecutive_errors,
                        "Error cleaning expired tasks"
                    );

                    if consecutive_errors >= 5 {
                        warn!(
                            consecutive_errors = consecutive_errors,
                            "Multiple consecutive task cleanup failures"
                        );
                    }
                }
            }
        }
    }

    async fn clean_expired_tasks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let span = info_span!("clean_expired_tasks");

        async {
            info!("Running cleanup of expired tasks");

            match self.task_repository.delete_expired_tasks().await {
                Ok(count) => {
                    if count > 0 {
                        info!(
                            deleted_count = count,
                            "Successfully cleaned up expired tasks"
                        );
                    } else {
                        debug!("No expired tasks to clean up");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to clean up expired tasks");
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            }

            Ok(())
        }
        .instrument(span)
        .await
    }
}
