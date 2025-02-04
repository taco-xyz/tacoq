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
                is_error, output_data, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                input_data = EXCLUDED.input_data,
                started_at = EXCLUDED.started_at,
                completed_at = EXCLUDED.completed_at,
                ttl = EXCLUDED.ttl,
                assigned_to = EXCLUDED.assigned_to,
                is_error = EXCLUDED.is_error,
                output_data = EXCLUDED.output_data,
                updated_at = NOW()
            RETURNING *
            "#,
            t.id,
            t.task_kind_name,
            t.worker_kind_name,
            t.input_data,
            t.started_at,
            t.completed_at,
            t.ttl,
            t.assigned_to,
            t.is_error,
            t.output_data,
            t.created_at,
            t.updated_at
        )
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_id<'e, E>(&self, executor: E, id: &Uuid) -> Result<Task, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", id)
            .fetch_one(executor)
            .await
    }
}

#[async_trait]
impl TaskRepository for PgTaskRepository {
    #[instrument(skip(self, id), fields(id = %id))]
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Task, sqlx::Error> {
        self.find_by_id(&self.core.pool, id).await
    }

    #[instrument(skip(self, task))]
    async fn update_task(&self, task: &Task) -> Result<Task, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let existing = self.find_by_id(&mut *tx, &task.id).await.ok();

        let task_to_save = match existing {
            Some(existing) => {
                match (existing.status(), task.status()) {
                    // Don't override completed tasks
                    (TaskStatus::Completed, _) => existing,

                    // Processing overrides pending
                    (TaskStatus::Pending, TaskStatus::Processing) => task.clone(),
                    (TaskStatus::Processing, TaskStatus::Pending) => existing,

                    // Default to the new task
                    _ => task.clone(),
                }
            }
            None => task.clone(),
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
    use crate::repo::PgRepositoryCore;
    use crate::testing::test::init_test_logger;

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    /// Creates a test task
    fn get_test_task() -> Task {
        Task::new(
            Some(Uuid::new_v4()),
            "TaskKindName",
            "WorkerKindName",
            None,
            None,
            None,
            Utc::now(),
            None,
            None,
            None,
            None,
        )
    }

    /// Creates a task and then retrieves it by id
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn create_and_get_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));

        let task = get_test_task();

        let saved = repo.update_task(&task).await.unwrap();

        assert_eq!(saved.id, task.id, "Created Task ID should match");

        let retrieved = repo.get_task_by_id(&task.id).await.unwrap();

        assert_eq!(retrieved.id, task.id, "Retrieved Task ID should match");
    }

    /// Tests task updating logic
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn update_task_progressive_status(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool));
        let task = get_test_task();
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
        let task = repo.get_task_by_id(&Uuid::new_v4()).await;
        assert!(task.is_err());
    }
}
