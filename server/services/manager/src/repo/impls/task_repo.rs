use async_trait::async_trait;
use common::models::{Task, TaskStatus};
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
}

#[async_trait]
impl TaskRepository for PgTaskRepository {
    #[instrument(skip(self, input_data), fields(task_kind_name = %task_kind_name))]
    async fn create_task(
        &self,
        task_kind_name: &str,
        input_data: Option<serde_json::Value>,
    ) -> Result<Task, sqlx::Error> {
        let task = Task::new(task_kind_name, input_data);
        task.save(&self.core.pool).await
    }

    #[instrument(skip(self, task_id, worker_id), fields(task_id = %task_id, worker_id = %worker_id))]
    async fn assign_task_to_worker(
        &self,
        task_id: &Uuid,
        worker_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;
        let mut task = Task::find_by_id(&mut *tx, task_id).await?;
        task.mark_processing(*worker_id);
        task.save(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, id), fields(id = %id))]
    async fn get_task_by_id(&self, id: &Uuid) -> Result<Task, sqlx::Error> {
        Task::find_by_id(&self.core.pool, id).await
    }

    #[instrument(skip(self, task_id, status), fields(task_id = %task_id, status = %status))]
    async fn update_task_status(
        &self,
        task_id: &Uuid,
        status: TaskStatus,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;
        let mut task = Task::find_by_id(&mut *tx, task_id).await?;
        task.set_status(status);
        task.save(&mut *tx).await?;
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
        let mut task = Task::find_by_id(&mut *tx, task_id).await?;
        task.mark_completed(error, true);
        let task = task.save(&mut *tx).await?;
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
        let mut task = Task::find_by_id(&mut *tx, task_id).await?;
        task.mark_completed(output, false);
        let task = task.save(&mut *tx).await?;
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
    use crate::repo::{PgRepositoryCore, PgWorkerRepository, WorkerRepository};
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
        let task_kind_name = "Test Task";

        let input = serde_json::json!({"test": "data"});
        let task = repo
            .create_task(task_kind_name, Some(input.clone()))
            .await
            .unwrap();

        assert_eq!(task.task_kind_name, task_kind_name);
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
        let worker_repo = PgWorkerRepository::new(core);
        let task_kind_name = "TaskKindTest";

        let task = repo.create_task(task_kind_name, None).await.unwrap();
        let worker_id = Uuid::new_v4();
        worker_repo
            .register_worker("Test Worker", "WorkerKindTest")
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
        let task2 = repo.create_task(task_kind_name, None).await.unwrap();
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

        let task = repo.create_task("TaskKindTest", None).await.unwrap();
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

        let task = repo.create_task("TaskKindTest", None).await.unwrap();
        assert_eq!(task.input_data, None);
    }

    /// Creates a task and then retrieves its results, which should be empty (no results yet)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_task_results_empty(pool: PgPool) {
        let repo = PgTaskRepository::new(PgRepositoryCore::new(pool.clone()));

        let task = repo.create_task("TaskKindTest", None).await.unwrap();
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
        let task = repo.create_task("TaskKindTest", None).await.unwrap();

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
        let worker_repo = PgWorkerRepository::new(core);

        let task = repo.create_task("TaskKindTest", None).await.unwrap();
        let worker_id = Uuid::new_v4();
        worker_repo
            .register_worker("Test Worker", "Test Worker")
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
