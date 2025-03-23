use crate::models::{Task, TaskAssignmentUpdate, TaskCompletedUpdate, TaskRunningUpdate};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::repo::PgRepositoryCore;

#[derive(Clone, Debug)]
pub struct TaskRepository {
    core: PgRepositoryCore,
}

impl TaskRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }

    // Basic CRUD

    #[instrument(skip(self, id), fields(id = %id))]
    pub async fn get_task_by_id(&self, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
        debug!(task_id = %id, "Getting task by ID");
        sqlx::query_as!(
            Task,
            r#"SELECT 
                id, 
                task_kind_name AS task_kind, 
                input_data, 
                output_data, 
                is_error, 
                started_at, 
                completed_at, 
                ttl_duration,
                worker_kind_name AS worker_kind, 
                executed_by, 
                created_at, 
                updated_at,
                priority,
                otel_ctx_carrier
            FROM tasks WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.core.pool)
        .await
    }

    #[instrument(skip(self))]
    pub async fn create_task(&self, task: &Task) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                id, task_kind_name, worker_kind_name, input_data, output_data,
                executed_by, is_error, priority, otel_ctx_carrier, ttl_duration,
                started_at, completed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            task.id,
            task.task_kind,
            task.worker_kind,
            task.input_data,
            task.output_data,
            task.executed_by,
            task.is_error,
            task.priority,
            task.otel_ctx_carrier,
            task.ttl_duration,
            task.started_at,
            task.completed_at,
            task.created_at,
            task.updated_at
        )
        .execute(&self.core.pool)
        .await?;
        Ok(())
    }

    // Update Consumer

    #[instrument(skip(self))]
    pub async fn update_task_from_assignment_update(
        &self,
        update: &TaskAssignmentUpdate,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                id, task_kind_name, worker_kind_name, input_data, 
                ttl_duration, priority, created_at, otel_ctx_carrier
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                task_kind_name = COALESCE(tasks.task_kind_name, EXCLUDED.task_kind_name),
                worker_kind_name = COALESCE(tasks.worker_kind_name, EXCLUDED.worker_kind_name), 
                input_data = COALESCE(tasks.input_data, EXCLUDED.input_data),
                ttl_duration = COALESCE(tasks.ttl_duration, EXCLUDED.ttl_duration),
                priority = COALESCE(tasks.priority, EXCLUDED.priority),
                created_at = COALESCE(tasks.created_at, EXCLUDED.created_at),
                otel_ctx_carrier = COALESCE(tasks.otel_ctx_carrier, EXCLUDED.otel_ctx_carrier)
            "#,
            update.id,
            update.task_kind,
            update.worker_kind,
            update.input_data,
            update.ttl_duration,
            update.priority,
            update.created_at,
            serde_json::to_value(&update.otel_ctx_carrier).unwrap()
        )
        .execute(&self.core.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update_task_from_completed_update(
        &self,
        update: &TaskCompletedUpdate,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                id, completed_at, output_data, is_error
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET
                completed_at = COALESCE(tasks.completed_at, EXCLUDED.completed_at),
                output_data = COALESCE(tasks.output_data, EXCLUDED.output_data),
                is_error = COALESCE(tasks.is_error, EXCLUDED.is_error)
            "#,
            update.id,
            update.completed_at,
            update.output_data,
            update.is_error
        )
        .execute(&self.core.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update_task_from_running_update(
        &self,
        update: &TaskRunningUpdate,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                id, started_at, executed_by
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET
                started_at = COALESCE(tasks.started_at, EXCLUDED.started_at),
                executed_by = COALESCE(tasks.executed_by, EXCLUDED.executed_by)
            "#,
            update.id,
            update.started_at,
            update.executed_by
        )
        .execute(&self.core.pool)
        .await?;
        Ok(())
    }

    // Cleanup

    #[instrument(skip(self))]
    pub async fn delete_expired_tasks(&self) -> Result<u64, sqlx::Error> {
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
    use std::collections::HashMap;

    use chrono::Local;
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
        Task::new("TaskKindName", "WorkerKindName", 0, 0)
            .with_input_data(vec![1, 2, 3])
            .with_output_data(vec![4, 5, 6])
            .with_error(false)
    }

    /// Creates a task and then retrieves it by id
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn create_and_get_task(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool.clone()));

        let task = get_test_task();

        repo.create_task(&task).await.unwrap();
        let new_task = repo.get_task_by_id(&task.id).await.unwrap().unwrap();

        assert_eq!(new_task.id, task.id, "Created Task ID should match");
        let retrieved = repo.get_task_by_id(&task.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, task.id, "Retrieved Task ID should match");
    }

    /// Tests task updating logic
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_task_assignment_update(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let id = Uuid::new_v4();
        let now = Local::now().naive_local();

        let update = TaskAssignmentUpdate {
            id,
            task_kind: "TestKind".to_string(),
            worker_kind: "TestWorker".to_string(),
            created_at: now,
            input_data: Some(vec![1, 2, 3]),
            priority: 1,
            ttl_duration: 60,
            otel_ctx_carrier: HashMap::new(),
        };

        repo.update_task_from_assignment_update(&update)
            .await
            .unwrap();

        let task = repo.get_task_by_id(&id).await.unwrap().unwrap();
        assert_eq!(task.task_kind, Some("TestKind".to_string()));
        assert_eq!(task.worker_kind, Some("TestWorker".to_string()));
        assert_eq!(task.input_data, Some(vec![1, 2, 3]));
        assert_eq!(task.ttl_duration, Some(60));
        assert_eq!(task.priority, Some(1));
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_task_running_update(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let id = Uuid::new_v4();
        let now = Local::now().naive_local();

        let update = TaskRunningUpdate::new(id, now, "worker-1".to_string());

        repo.update_task_from_running_update(&update).await.unwrap();
        let task = repo.get_task_by_id(&id).await.unwrap();
        println!("task: {:?}", task);

        let task = task.unwrap();
        assert_eq!(
            task.started_at.unwrap().and_utc().timestamp_micros(),
            now.and_utc().timestamp_micros()
        );
        assert_eq!(task.executed_by, Some("worker-1".to_string()));
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_task_completed_update(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let id = Uuid::new_v4();
        let now = Local::now().naive_local();

        let update = TaskCompletedUpdate::new(id, now, Some(vec![4, 5, 6]), 0);

        repo.update_task_from_completed_update(&update)
            .await
            .unwrap();

        let task = repo.get_task_by_id(&id).await.unwrap().unwrap();
        assert_eq!(
            task.completed_at.unwrap().and_utc().timestamp_micros(),
            now.and_utc().timestamp_micros()
        );
        assert_eq!(task.output_data, Some(vec![4, 5, 6]));
        assert_eq!(task.is_error, Some(0));
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_task_full_lifecycle(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool.clone()));
        let id = Uuid::new_v4();
        let now = Local::now().naive_local();

        // 1. Assignment
        let assignment = TaskAssignmentUpdate {
            id,
            task_kind: "test_task".to_string(),
            worker_kind: "test_worker".to_string(),
            created_at: now,
            input_data: Some(vec![1, 2, 3]),
            priority: 1,
            ttl_duration: 3600000000, // 1 hour in microseconds
            otel_ctx_carrier: HashMap::new(),
        };
        repo.update_task_from_assignment_update(&assignment)
            .await
            .unwrap();

        // 2. Running
        let running = TaskRunningUpdate::new(id, now, "worker-1".to_string());
        repo.update_task_from_running_update(&running)
            .await
            .unwrap();

        // 3. Completed
        let completed = TaskCompletedUpdate::new(id, now, Some(vec![4, 5, 6]), 0);
        repo.update_task_from_completed_update(&completed)
            .await
            .unwrap();

        // Verify final state
        let task = repo.get_task_by_id(&id).await.unwrap().unwrap();
        assert_eq!(task.task_kind, Some("test_task".to_string()));
        assert_eq!(task.worker_kind, Some("test_worker".to_string()));
        assert_eq!(task.input_data, Some(vec![1, 2, 3]));
        assert_eq!(task.ttl_duration, Some(3600000000));
        assert_eq!(task.priority, Some(1));
        assert_eq!(
            task.started_at.unwrap().and_utc().timestamp_micros(),
            now.and_utc().timestamp_micros()
        );
        assert_eq!(task.executed_by, Some("worker-1".to_string()));
        assert_eq!(
            task.completed_at.unwrap().and_utc().timestamp_micros(),
            now.and_utc().timestamp_micros()
        );
        assert_eq!(task.output_data, Some(vec![4, 5, 6]));
        assert_eq!(task.is_error, Some(0));
    }

    /// Attempts to retrieve a non-existent task (should fail)
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn get_nonexistent_task(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool));
        let task = repo.get_task_by_id(&Uuid::new_v4()).await.unwrap();
        assert!(task.is_none());
    }

    // Tests task cleanup on expired ttl
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn cleanup_expired_tasks(pool: PgPool) {
        let repo = TaskRepository::new(PgRepositoryCore::new(pool.clone()));

        // Set completed at one day ago
        let mut task = get_test_task();
        task.completed_at = Some(Local::now().naive_local() - chrono::Duration::days(1));

        repo.create_task(&task).await.unwrap();

        let count = repo.delete_expired_tasks().await.unwrap();
        assert_eq!(count, 1, "One task should be deleted");

        let count = repo.delete_expired_tasks().await.unwrap();
        assert_eq!(count, 0, "No more tasks should be deleted");
    }
}
