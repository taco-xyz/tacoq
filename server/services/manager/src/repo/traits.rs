use std::{fmt::Debug, time::SystemTime};

use async_trait::async_trait;
use common::models::{Task, TaskStatus, Worker, WorkerKind};
use uuid::Uuid;

/// Repository trait for managing task records in the database
///
/// Provides methods for creating new tasks and retrieving existing tasks by their ID.
/// Tasks represent units of work that can be assigned to and processed by workers.
#[async_trait]
pub trait TaskRepository: Send + Sync + Clone + Debug {
    /// Create a new task in the database
    async fn create_task(
        &self,
        task_kind_name: &str,
        worker_kind_name: &str,
        input_data: Option<serde_json::Value>,
    ) -> Result<Task, sqlx::Error>;

    /// Assign a task to a worker
    async fn assign_task_to_worker(
        &self,
        task_id: &Uuid,
        worker_id: &Uuid,
    ) -> Result<(), sqlx::Error>;

    /// Get a task by its ID
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Task, sqlx::Error>;

    /// Update the status of a task
    async fn update_task_status(
        &self,
        task_id: &Uuid,
        status: TaskStatus,
    ) -> Result<(), sqlx::Error>;

    /// Upload an error result for a task, marking it as failed
    async fn upload_task_error(
        &self,
        task_id: &Uuid,
        error: serde_json::Value,
    ) -> Result<Task, sqlx::Error>;

    /// Upload a successful result for a task, marking it as completed
    async fn upload_task_result(
        &self,
        task_id: &Uuid,
        output: serde_json::Value,
    ) -> Result<Task, sqlx::Error>;
}

/// Repository trait for managing worker records in the database
///
/// Provides methods for registering and managing workers that can process tasks.
#[async_trait]
pub trait WorkerRepository: Clone {
    /// Register a new worker with its supported task types
    async fn register_worker(
        &self,
        name: &str,
        worker_kind_name: &str,
    ) -> Result<Worker, sqlx::Error>;

    /// Get a worker by ID
    async fn _get_worker_by_id(&self, id: &Uuid) -> Result<Worker, sqlx::Error>;

    /// Get all registered workers
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error>;

    /// Record a heartbeat for a worker
    async fn _record_heartbeat(&self, worker_id: &Uuid) -> Result<(), sqlx::Error>;

    /// Get the latest heartbeat for a worker
    async fn _get_latest_heartbeat(&self, worker_id: &Uuid) -> Result<SystemTime, sqlx::Error>;
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
        exchange: &str,
        queue: &str,
    ) -> Result<WorkerKind, sqlx::Error>;
}
