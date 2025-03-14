use crate::models::{Task, TaskStatus};
use async_trait::async_trait;
use sqlx::{Executor, Postgres};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::repo::{PgRepositoryCore, TaskRepository};

#[derive(Clone, Debug)]
pub struct PgTaskRepository {
    core: PgRepositoryCore,
}

impl PgTaskRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }

    pub async fn save<'e, E>(&self, executor: E, t: &Task) -> Result<Task, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(task_id = %t.id, task_kind = %t.task_kind, status = %t.status, "Saving task");
        sqlx::query_as!(
            Task,
            r#"
            INSERT INTO tasks (
                id, task_kind_name, worker_kind_name, input_data, started_at, completed_at, ttl_duration, executed_by,
                is_error, output_data, created_at, updated_at, status, priority, otel_ctx_carrier
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (id) DO UPDATE SET
                input_data = EXCLUDED.input_data,
                started_at = EXCLUDED.started_at,
                completed_at = EXCLUDED.completed_at,
                ttl_duration = EXCLUDED.ttl_duration,
                executed_by = EXCLUDED.executed_by,
                is_error = EXCLUDED.is_error,
                output_data = EXCLUDED.output_data,
                status = EXCLUDED.status,
                priority = EXCLUDED.priority,
                updated_at = NOW(),
                otel_ctx_carrier = EXCLUDED.otel_ctx_carrier
            RETURNING 
                id, 
                task_kind_name AS "task_kind!", 
                input_data, 
                output_data, 
                is_error, 
                started_at, 
                completed_at, 
                ttl_duration,
                worker_kind_name AS "worker_kind!", 
                executed_by, 
                created_at, 
                updated_at,
                status,
                priority,
                otel_ctx_carrier
            "#,
            t.id,
            t.task_kind,
            t.worker_kind,
            t.input_data,
            t.started_at,
            t.completed_at,
            t.ttl_duration,
            t.executed_by,
            t.is_error,
            t.output_data,
            t.created_at,
            t.updated_at,
            t.status.to_string(),
            t.priority,
            t.otel_ctx_carrier,
        )
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_id<'e, E>(
        &self,
        executor: E,
        id: &Uuid,
    ) -> Result<Option<Task>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(task_id = %id, "Finding task by ID");
        sqlx::query_as!(
            Task,
            r#"SELECT 
                id, 
                task_kind_name AS "task_kind!", 
                input_data, 
                output_data, 
                is_error, 
                started_at, 
                completed_at, 
                ttl_duration,
                worker_kind_name AS "worker_kind!", 
                executed_by, 
                created_at, 
                updated_at,
                status,
                priority,
                otel_ctx_carrier
            FROM tasks WHERE id = $1"#,
            id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn delete<'e, E>(&self, executor: E, id: &Uuid) -> Result<(), sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(task_id = %id, "Deleting task");
        sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
            .execute(executor)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TaskRepository for PgTaskRepository {
    #[instrument(skip(self, id), fields(id = %id))]
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
        debug!(task_id = %id, "Getting task by ID");
        let result = self.find_by_id(&self.core.pool, id).await;

        match &result {
            Ok(Some(task)) => info!(
                task_id = %id,
                task_kind = %task.task_kind,
                status = %task.status,
                "Task found"
            ),
            Ok(None) => debug!(task_id = %id, "Task not found"),
            Err(e) => error!(
                task_id = %id,
                error = %e,
                "Error fetching task"
            ),
        }

        result
    }

    #[instrument(skip(self, task), fields(task_id = %task.id, task_kind = %task.task_kind, status = %task.status))]
    async fn update_task(&self, task: &Task) -> Result<Task, sqlx::Error> {
        info!(task_id = %task.id, "Updating task");
        let mut tx = match self.core.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!(task_id = %task.id, error = %e, "Failed to start transaction");
                return Err(e);
            }
        };

        let existing = self.find_by_id(&mut *tx, &task.id).await?;

        let task_to_save = if let Some(existing) = existing {
            debug!(
                task_id = %task.id,
                existing_status = %existing.status,
                new_status = %task.status,
                "Determining task priority"
            );

            match (existing.status(), task.status()) {
                // Don't override completed tasks
                (TaskStatus::Completed, _) => {
                    debug!(task_id = %task.id, "Task already completed, not updating");
                    existing
                }

                // Processing overrides pending
                (TaskStatus::Pending, TaskStatus::Processing) => {
                    debug!(task_id = %task.id, "Updating from Pending to Processing");
                    task.clone()
                }
                (TaskStatus::Processing, TaskStatus::Pending) => {
                    debug!(task_id = %task.id, "Already Processing, ignoring Pending update");
                    existing
                }

                // Default to the new task
                _ => {
                    debug!(task_id = %task.id, "Updating task with new data");
                    task.clone()
                }
            }
        } else {
            info!(task_id = %task.id, task_kind = %task.task_kind, "Creating new task");
            task.clone()
        };

        let saved = match self.save(&mut *tx, &task_to_save).await {
            Ok(saved) => saved,
            Err(e) => {
                error!(task_id = %task.id, error = %e, "Failed to save task");
                return Err(e);
            }
        };

        if let Err(e) = tx.commit().await {
            error!(task_id = %task.id, error = %e, "Failed to commit transaction");
            return Err(e);
        }

        info!(task_id = %task.id, "Task successfully updated");
        Ok(saved)
    }

    #[instrument(skip(self, id), fields(task_id = %id))]
    async fn delete_task(&self, id: &Uuid) -> Result<(), sqlx::Error> {
        info!(task_id = %id, "Deleting task");
        match self.delete(&self.core.pool, id).await {
            Ok(_) => {
                info!(task_id = %id, "Task successfully deleted");
                Ok(())
            }
            Err(e) => {
                error!(task_id = %id, error = %e, "Failed to delete task");
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    async fn delete_expired_tasks(&self) -> Result<u64, sqlx::Error> {
        info!("Cleaning up expired tasks");
        let now = chrono::Utc::now().naive_utc();

        let result = match sqlx::query!(
            r#"DELETE FROM tasks
                WHERE completed_at IS NOT NULL AND completed_at + interval '1 second' * ttl_duration < $1
            "#,
            now,
        )
        .execute(&self.core.pool)
        .await
        {
            Ok(result) => result,
            Err(e) => {
                error!(error = %e, "Failed to delete expired tasks");
                return Err(e);
            }
        };

        let count = result.rows_affected();
        info!(deleted_count = count, "Deleted expired tasks");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::repo::{PgRepositoryCore, PgWorkerKindRepository, WorkerKindRepository};
    use crate::testing::test::init_test_logger;

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    /// Creates a test task
    fn get_test_task() -> Task {
        Task::new("TaskKindName", "WorkerKindName", 0, 0)
            .with_input_data(vec![1, 2, 3])
            .with_output_data(vec![4, 5, 6])
            .with_error(false)
            .with_status(TaskStatus::Pending)
    }

    /// Creates a task and then retrieves it by id
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn create_and_get_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let worker_kind_repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool.clone()));

        let task = get_test_task();
        worker_kind_repo
            .get_or_create_worker_kind(&task.worker_kind)
            .await
            .unwrap();

        let saved = repo.update_task(&task).await.unwrap();

        assert_eq!(saved.id, task.id, "Created Task ID should match");

        let retrieved = repo.get_task_by_id(&task.id).await.unwrap().unwrap();

        assert_eq!(retrieved.id, task.id, "Retrieved Task ID should match");
    }

    /// Tests task updating logic
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn update_task_progressive_status(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let worker_kind_repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool.clone()));

        let task = get_test_task();

        worker_kind_repo
            .get_or_create_worker_kind(&task.worker_kind)
            .await
            .unwrap();

        let saved = repo.update_task(&task).await.unwrap();
        assert_eq!(saved.id, task.id, "Created Task ID should match");

        // Simulate that the task is now in progress and try to update it
        let mut task = task.clone();

        task.status = TaskStatus::Processing;

        let updated = repo.update_task(&task).await.unwrap();
        assert_eq!(updated.id, task.id, "Updated Task ID should match");
        assert_eq!(
            updated.status,
            TaskStatus::Processing,
            "Task was set to Processing, so it should be in progress",
        );

        // Now attempt to set it to pending again and check if it does anything
        let mut task = task.clone();
        task.status = TaskStatus::Pending;
        let updated = repo.update_task(&task).await.unwrap();
        assert_eq!(updated.id, task.id, "Updated Task ID should match");
        assert_eq!(
            updated.status,
            TaskStatus::Processing,
            "Task was already updated to Processing, so status should not change back Pending"
        );
    }

    /// Attempts to retrieve a non-existent task (should fail)
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]

    async fn get_nonexistent_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool));
        let task = repo.get_task_by_id(&Uuid::new_v4()).await.unwrap();
        assert!(task.is_none());
    }

    // Tests task deletion
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn delete_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let worker_kind_repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool.clone()));

        let task = get_test_task();
        worker_kind_repo
            .get_or_create_worker_kind(&task.worker_kind)
            .await
            .unwrap();

        let saved = repo.update_task(&task).await.unwrap();
        assert_eq!(saved.id, task.id, "Created Task ID should match");

        repo.delete_task(&task.id).await.unwrap();

        let retrieved = repo.get_task_by_id(&task.id).await.unwrap();
        assert!(retrieved.is_none(), "Task should be deleted");
    }

    // Tests task cleanup on expired ttl
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn cleanup_expired_tasks(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let worker_kind_repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool.clone()));

        let mut task = get_test_task();

        // Set completed at one day ago
        task.completed_at = Some(Local::now().naive_local() - chrono::Duration::days(1));

        worker_kind_repo
            .get_or_create_worker_kind(&task.worker_kind)
            .await
            .unwrap();

        let saved = repo.update_task(&task).await.unwrap();
        assert_eq!(saved.id, task.id, "Created Task ID should match");

        let count = repo.delete_expired_tasks().await.unwrap();
        assert_eq!(count, 1, "One task should be deleted");

        let count = repo.delete_expired_tasks().await.unwrap();
        assert_eq!(count, 0, "No more tasks should be deleted");
    }
}
