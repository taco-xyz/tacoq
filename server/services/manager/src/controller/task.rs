use crate::repo::worker_kind_repo::PgWorkerKindRepository;
use crate::repo::{
    PgTaskRepository, PgWorkerRepository, TaskRepository, WorkerKindRepository, WorkerRepository,
};
use common::brokers::core::BrokerConsumer;
use common::models::Task;
use tracing::{error, info, info_span, warn, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskController {
    consumer: Arc<dyn BrokerConsumer<Task>>,
    worker_repository: PgWorkerRepository,
    worker_kind_repository: PgWorkerKindRepository,
    task_repository: PgTaskRepository,
}

impl TaskController {
    pub async fn new(
        consumer: Arc<dyn BrokerConsumer<Task>>,
        worker_repository: PgWorkerRepository,
        worker_kind_repository: PgWorkerKindRepository,
        task_repository: PgTaskRepository,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            consumer,
            worker_repository,
            worker_kind_repository,
            task_repository,
        })
    }

    async fn consume_task(&self, task: Task) {
        info!("Starting to process task: {:?}", task);

        let span = info_span!("consume_task", task_id = %task.id);
        let context = task.context();
        span.set_parent(context.clone());
        async {
            let kind = match self
                .worker_kind_repository
                .get_or_create_worker_kind(&task.worker_kind)
                .await
            {
                Ok(kind) => kind,
                Err(e) => {
                    error!("Failed to process worker kind: {}", e);
                    return;
                }
            };
            info!("Worker kind processed successfully: {:?}", kind);

            let task = match self.task_repository.update_task(&task).await {
                Ok(task) => task,
                Err(e) => {
                    error!("Failed to update task: {}", e);
                    return;
                }
            };
            info!("Task updated successfully: {:?}", task);

            if let Some(assigned_to) = task.assigned_to {
                match self
                    .worker_repository
                    .update_worker(assigned_to, &kind.name)
                    .await
                {
                    Ok(_) => info!("Worker updated successfully: {}", assigned_to),
                    Err(e) => warn!("Failed to update worker {}: {}", assigned_to, e),
                }
            }

            info!("Task processing completed successfully");
        }
        .instrument(span)
        .await;
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        // This clone is needed so the self object can live inside the closure due to move
        let this = self.clone();

        self.consumer
            .consume_messages(Box::new(move |task| {
                // Same thing here, we need to clone the self object so it can live inside the closure for 'static lifetime
                let this = this.clone();
                Box::pin(async move {
                    this.consume_task(task).await;
                })
            }))
            .await
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.consumer.shutdown().await
    }
}
