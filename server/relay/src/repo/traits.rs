use std::{fmt::Debug, time::SystemTime};

use crate::models::{Task, Worker, WorkerKind};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for managing task records in the database
///
/// Provides methods for creating new tasks and retrieving existing tasks by their ID.
/// Tasks represent units of work that can be assigned to and processed by workers.
#[async_trait]
pub trait TaskRepository: Send + Sync + Clone + Debug {
    /// Get a task by its ID
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Option<Task>, sqlx::Error>;

    /// Update a task. If it doesn't already exist, it is created. Whether or
    /// not the task is updated is based on the status of the task, where the
    /// priority order of statuses is:
    /// 1. `COMPLETED`
    /// 3. `IN_PROGRESS`
    /// 4. `PENDING`
    /// 5. New Task (not a status)
    ///
    /// This is done because in a distributed system, the events may arrive
    /// out of order.
    async fn update_task(&self, task: &Task) -> Result<Task, sqlx::Error>;

    /// Delete a task by its ID
    async fn delete_task(&self, id: &Uuid) -> Result<(), sqlx::Error>;

    /// Delete all tasks that have the ttl expired
    async fn delete_expired_tasks(&self) -> Result<u64, sqlx::Error>;
}

/// Repository trait for managing worker records in the database
///
/// Provides methods for registering and managing workers that can process tasks.
#[async_trait]
pub trait WorkerRepository: Clone {
    /// Register a new worker with its supported task types
    async fn update_worker(
        &self,
        name: &str,
        worker_kind_name: &str,
    ) -> Result<Worker, sqlx::Error>;

    /// Get a worker by name
    async fn _get_worker_by_name(&self, name: &str) -> Result<Option<Worker>, sqlx::Error>;

    /// Get all registered workers
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error>;

    /// Get the latest heartbeat for a worker
    async fn _get_latest_heartbeat(&self, name: &str) -> Result<SystemTime, sqlx::Error>;
}

/// Repository trait for managing worker kind records in the database
///
/// Provides methods for registering and managing worker kinds that workers can be classified as.
#[async_trait]
pub trait WorkerKindRepository: Clone {
    // Get a worker kind by name
    async fn get_or_create_worker_kind(
        &self,
        name: &str,
        // exchange: &str,
        // queue: &str,
    ) -> Result<WorkerKind, sqlx::Error>;
}
