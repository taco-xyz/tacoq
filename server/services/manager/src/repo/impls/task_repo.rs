use async_trait::async_trait;
use common::models::{Task, TaskStatus};
use sqlx::{Executor, Postgres};
use tracing::instrument;
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
        sqlx::query_as!(
            Task,
            r#"
            INSERT INTO tasks (
                id, task_kind_name, worker_kind_name, input_data, started_at, completed_at, ttl, assigned_to,
                is_error, output_data, created_at, updated_at, status, priority, otel_ctx_carrier
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (id) DO UPDATE SET
                input_data = EXCLUDED.input_data,
                started_at = EXCLUDED.started_at,
                completed_at = EXCLUDED.completed_at,
                ttl = EXCLUDED.ttl,
                assigned_to = EXCLUDED.assigned_to,
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
                status,
                priority, 
                started_at, 
                completed_at, 
                ttl, 
                worker_kind_name AS "worker_kind!", 
                assigned_to, 
                created_at, 
                updated_at,
                otel_ctx_carrier
            "#,
            t.id,
            t.task_kind,
            t.worker_kind,
            t.input_data,
            t.started_at,
            t.completed_at,
            t.ttl,
            t.assigned_to,
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
                ttl, 
                worker_kind_name AS "worker_kind!", 
                assigned_to, 
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
}

#[async_trait]
impl TaskRepository for PgTaskRepository {
    #[instrument(skip(self, id), fields(id = %id))]
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
        self.find_by_id(&self.core.pool, id).await
    }

    #[instrument(skip(self, task), parent_context = %task.otel_ctx_carrier)]
    async fn update_task(&self, task: &Task) -> Result<Task, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let existing = self.find_by_id(&mut *tx, &task.id).await?;

        let task_to_save = if let Some(existing) = existing {
            match (existing.status(), task.status()) {
                // Don't override completed tasks
                (TaskStatus::Completed, _) => existing,

                // Processing overrides pending
                (TaskStatus::Pending, TaskStatus::Processing) => task.clone(),
                (TaskStatus::Processing, TaskStatus::Pending) => existing,

                // Default to the new task
                _ => task.clone(),
            }
        } else {
            task.clone()
        };

        let saved = self.save(&mut *tx, &task_to_save).await?;
        tx.commit().await?;
        Ok(saved)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::types::chrono::Utc;
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
        Task::new("TaskKindName", "WorkerKindName", 0)
            .with_input_data(vec![1, 2, 3])
            .with_output_data(vec![4, 5, 6])
            .with_error(false)
            .with_status(TaskStatus::Pending)
    }

    /// Creates a task and then retrieves it by id
    #[sqlx::test(migrator = "common::MIGRATOR")]
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
    #[sqlx::test(migrator = "common::MIGRATOR")]
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
        task.started_at = Some(Utc::now());
        let updated = repo.update_task(&task).await.unwrap();
        assert_eq!(updated.id, task.id, "Updated Task ID should match");
        assert!(
            updated.started_at.is_some(),
            "Task was created with started_at = true, so it should be in progress",
        );

        // Now attempt to set it to pending again and check if it does anything
        let mut task = task.clone();
        task.started_at = None;
        let updated = repo.update_task(&task).await.unwrap();
        assert_eq!(updated.id, task.id, "Updated Task ID should match");
        assert!(
            updated.started_at.is_some(),
            "Task was updated with started_at = None, but the update should not go through"
        );
    }

    /// Attempts to retrieve a non-existent task (should fail)
    #[sqlx::test(migrator = "common::MIGRATOR")]

    async fn get_nonexistent_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool));
        let task = repo.get_task_by_id(&Uuid::new_v4()).await.unwrap();
        assert!(task.is_none());
    }
}
