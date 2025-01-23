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

    async fn save_task<'e, E>(&self, executor: E, task: &Task) -> Result<Task, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO tasks (
                id, task_kind_id, input_data, started_at, completed_at, ttl, assigned_to,
                is_error, output_data, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
        )
        .bind(task.id)
        .bind(task.task_kind_id)
        .bind(&task.input_data)
        .bind(task.started_at)
        .bind(task.completed_at)
        .bind(task.ttl)
        .bind(task.assigned_to)
        .bind(task.is_error)
        .bind(&task.output_data)
        .bind(task.created_at)
        .bind(task.updated_at)
        .fetch_one(executor)
        .await
    }

    async fn find_task<'e, E>(&self, executor: E, id: &Uuid) -> Result<Task, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
    }
}

#[async_trait]
impl TaskRepository for PgTaskRepository {
    #[instrument(skip(self, input_data), fields(task_kind_id = %task_kind_id))]
    async fn create_task(
        &self,
        task_kind_id: Uuid,
        input_data: Option<serde_json::Value>,
    ) -> Result<Task, sqlx::Error> {
        let task = Task::new(task_kind_id, input_data);
        self.save_task(&self.core.pool, &task).await
    }

    #[instrument(skip(self, task_id, worker_id), fields(task_id = %task_id, worker_id = %worker_id))]
    async fn assign_task_to_worker(
        &self,
        task_id: &Uuid,
        worker_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let mut task = self.find_task(&mut *tx, task_id).await?;
        task.mark_processing(*worker_id);
        self.save_task(&mut *tx, &task).await?;

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, id), fields(id = %id))]
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Task, sqlx::Error> {
        self.find_task(&self.core.pool, id).await
    }

    #[instrument(skip(self, task_id, status), fields(task_id = %task_id, status = %status))]
    async fn update_task_status(
        &self,
        task_id: &Uuid,
        status: TaskStatus,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let mut task = self.find_task(&mut *tx, task_id).await?;
        if status == TaskStatus::Completed {
            task.mark_completed();
        }
        self.save_task(&mut *tx, &task).await?;

        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, task_id, worker_id, error), fields(task_id = %task_id, worker_id = %worker_id))]
    async fn upload_task_error(
        &self,
        task_id: &Uuid,
        worker_id: &Uuid,
        error: serde_json::Value,
    ) -> Result<Task, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let mut task = self.find_task(&mut *tx, task_id).await?;
        task.set_error(*worker_id, error);
        let task = self.save_task(&mut *tx, &task).await?;

        tx.commit().await?;
        Ok(task)
    }

    #[instrument(skip(self, task_id, worker_id, output), fields(task_id = %task_id, worker_id = %worker_id))]
    async fn upload_task_result(
        &self,
        task_id: &Uuid,
        worker_id: &Uuid,
        output: serde_json::Value,
    ) -> Result<Task, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let mut task = self.find_task(&mut *tx, task_id).await?;
        task.set_result(*worker_id, output);
        let task = self.save_task(&mut *tx, &task).await?;

        tx.commit().await?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use common::models::TaskStatus;

    use super::*;
    use crate::repo::{
        PgRepositoryCore, PgTaskKindRepository, PgWorkerRepository, TaskKindRepository,
        WorkerRepository,
    };
    use crate::testing::test::init_test_logger;

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    /// Creates a task and then retrieves it by id
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn create_and_get_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();

        let input = serde_json::json!({"test": "data"});
        let task = repo
            .create_task(task_kind.id, Some(input.clone()))
            .await
            .unwrap();

        assert_eq!(task.task_kind_id, task_kind.id);
        assert_eq!(task.input_data, Some(input));
        assert_eq!(task.is_error, 0);
        assert_eq!(task.assigned_to, None);

        let retrieved = repo.get_task_by_id(&task.id).await.unwrap();
        assert_eq!(task.id, retrieved.id);
    }

    /// Creates a task and then uploads a result and an error
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn create_task_and_then_upload_error(pool: PgPool) {
        let core = PgRepositoryCore::new(pool.clone());
        let repo = PgTaskRepository::new(core.clone());
        let task_kind_repo = PgTaskKindRepository::new(core.clone());
        let worker_repo = PgWorkerRepository::new(core);

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();
        let task = repo.create_task(task_kind.id, None).await.unwrap();
        let worker_id = Uuid::new_v4();
        worker_repo
            .register_worker(
                worker_id,
                "Test Worker".to_string(),
                vec![task_kind.clone()],
            )
            .await
            .unwrap();

        // Test successful result
        let output = serde_json::json!({"result": "success"});
        let result = repo
            .upload_task_result(&task.id, &worker_id, output.clone())
            .await
            .unwrap();

        assert_eq!(result.id, task.id);
        assert_eq!(result.assigned_to, Some(worker_id));
        assert_eq!(result.output_data, Some(output));
        assert_eq!(result.is_error, 0);
        assert!(result.completed_at.is_some());

        // Test error result
        let task2 = repo.create_task(task_kind.id, None).await.unwrap();
        let error = serde_json::json!({"error": "failed"});
        let error_result = repo
            .upload_task_error(&task2.id, &worker_id, error.clone())
            .await
            .unwrap();

        assert_eq!(error_result.id, task2.id);
        assert_eq!(error_result.assigned_to, Some(worker_id));
        assert_eq!(error_result.output_data, Some(error));
        assert_eq!(error_result.is_error, 1);
        assert!(error_result.completed_at.is_some());
    }

    /// Tests that a task's status can be updated after creation
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_task_status_update(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();
        let task = repo.create_task(task_kind.id, None).await.unwrap();
        assert!(task.started_at.is_none());

        repo.update_task_status(&task.id, TaskStatus::Processing)
            .await
            .unwrap();
        let updated = repo.get_task_by_id(&task.id).await.unwrap();
        assert!(updated.started_at.is_some());

        repo.update_task_status(&task.id, TaskStatus::Completed)
            .await
            .unwrap();
        let completed = repo.get_task_by_id(&task.id).await.unwrap();
        assert!(completed.completed_at.is_some());
    }

    /// Creates a task without input data (should be allowed)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn create_task_without_input_data(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();
        let task = repo.create_task(task_kind.id, None).await.unwrap();
        assert_eq!(task.input_data, None);
    }

    /// Creates a task and then retrieves its results, which should be empty (no results yet)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_task_results_empty(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();
        let task = repo.create_task(task_kind.id, None).await.unwrap();
        let task = repo.get_task_by_id(&task.id).await.unwrap();
        assert!(task.output_data.is_none());
    }

    /// Attempts to retrieve a non-existent task (should fail)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_nonexistent_task(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool));
        let task = repo.get_task_by_id(&Uuid::new_v4()).await;
        assert!(task.is_err());
    }

    /// Creates a task and then updates its status through all possible transitions
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn status_transitions(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();
        let task = repo.create_task(task_kind.id, None).await.unwrap();

        // Test full lifecycle
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());

        repo.update_task_status(&task.id, TaskStatus::Processing)
            .await
            .unwrap();
        let task = repo.get_task_by_id(&task.id).await.unwrap();
        assert!(task.started_at.is_some());

        repo.update_task_status(&task.id, TaskStatus::Completed)
            .await
            .unwrap();
        let task = repo.get_task_by_id(&task.id).await.unwrap();
        assert!(task.completed_at.is_some());
    }

    /// Tests assigning a task to a worker
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_assign_task_to_worker(pool: PgPool) {
        let core = PgRepositoryCore::new(pool.clone());
        let repo = PgTaskRepository::new(core.clone());
        let task_kind_repo = PgTaskKindRepository::new(core.clone());
        let worker_repo = PgWorkerRepository::new(core);

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test Task")
            .await
            .unwrap();
        let task = repo.create_task(task_kind.id, None).await.unwrap();
        let worker_id = Uuid::new_v4();
        worker_repo
            .register_worker(
                worker_id,
                "Test Worker".to_string(),
                vec![task_kind.clone()],
            )
            .await
            .unwrap();

        repo.assign_task_to_worker(&task.id, &worker_id)
            .await
            .unwrap();
        let updated = repo.get_task_by_id(&task.id).await.unwrap();
        assert_eq!(updated.assigned_to, Some(worker_id));
        assert!(updated.started_at.is_some());
    }
}
