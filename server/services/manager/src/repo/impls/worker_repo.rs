use async_trait::async_trait;
use common::models::{TaskKind, Worker, WorkerHeartbeat, WorkerKind};
use sqlx::{Executor, Postgres};
use std::time::SystemTime;
use tracing::instrument;
use uuid::Uuid;

use crate::repo::{PgRepositoryCore, WorkerRepository};

#[derive(Clone)]
pub struct PgWorkerRepository {
    core: PgRepositoryCore,
}

impl PgWorkerRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }

    async fn save_worker<'e, E>(&self, executor: E, worker: &Worker) -> Result<Worker, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO workers (id, name, worker_kind_name, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE 
            SET name = $2,
                worker_kind_name = $3,
                active = $4
            RETURNING *
            "#,
        )
        .bind(worker.id)
        .bind(&worker.name)
        .bind(&worker.worker_kind_name)
        .bind(worker.created_at)
        .fetch_one(executor)
        .await
    }

    async fn find_worker<'e, E>(&self, executor: E, id: &Uuid) -> Result<Worker, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT * FROM workers WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(executor)
        .await
    }

    // Consider putting a limit here for pagination in the future
    async fn find_all_workers<'e, E>(&self, executor: E) -> Result<Vec<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT * FROM workers
            "#,
        )
        .fetch_all(executor)
        .await
    }

    async fn save_heartbeat<'e, E>(
        &self,
        executor: E,
        heartbeat: &WorkerHeartbeat,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO worker_heartbeats (worker_id, heartbeat_time, created_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(heartbeat.worker_id)
        .bind(heartbeat.heartbeat_time)
        .bind(heartbeat.created_at)
        .execute(executor)
        .await?;
        Ok(())
    }

    async fn get_latest_heartbeat<'e, E>(
        &self,
        executor: E,
        worker_id: &Uuid,
    ) -> Result<WorkerHeartbeat, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT * 
            FROM worker_heartbeats 
            WHERE worker_id = $1 
            ORDER BY heartbeat_time DESC 
            LIMIT 1
            "#,
        )
        .bind(worker_id)
        .fetch_one(executor)
        .await?
    }

    async fn save_worker_kind<'e, E>(
        &self,
        executor: E,
        worker_kind: &WorkerKind,
    ) -> Result<WorkerKind, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO worker_kinds (name, routing_key, queue_name, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (name) DO UPDATE 
            SET routing_key = $2,
                queue_name = $3
            RETURNING *
            "#,
        )
        .bind(&worker_kind.name)
        .bind(&worker_kind.routing_key)
        .bind(&worker_kind.queue_name)
        .bind(worker_kind.created_at)
        .fetch_one(executor)
        .await
    }

    async fn find_worker_kind_by_name<'e, E>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<Option<WorkerKind>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT * FROM worker_kinds WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(executor)
        .await
    }
}

#[async_trait]
impl WorkerRepository for PgWorkerRepository {
    #[instrument(skip(self, name, worker_kind_name), fields(name = %name, worker_kind_name = %worker_kind_name))]
    async fn register_worker(
        &self,
        name: &str,
        worker_kind_name: &str,
    ) -> Result<Worker, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let worker = Worker::new(name, worker_kind_name);
        let worker_kind = WorkerKind::new(worker_kind_name, worker_kind_name, worker_kind_name);

        self.save_worker_kind(&mut *tx, &worker_kind).await?;
        let worker = self.save_worker(&mut *tx, &worker).await?;

        tx.commit().await?;
        Ok(worker)
    }

    #[instrument(skip(self, id), fields(id = %id))]
    async fn _get_worker_by_id(&self, id: &Uuid) -> Result<Worker, sqlx::Error> {
        let worker = self.find_worker(&self.core.pool, id).await?;
        Ok(worker)
    }

    #[instrument(skip(self))]
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error> {
        let workers = self.find_all_workers(&self.core.pool).await?;
        Ok(workers)
    }

    #[instrument(skip(self, worker_id), fields(worker_id = %worker_id))]
    async fn _record_heartbeat(&self, worker_id: &Uuid) -> Result<(), sqlx::Error> {
        let heartbeat = WorkerHeartbeat::new(*worker_id);
        self.save_heartbeat(&self.core.pool, &heartbeat).await
    }

    #[instrument(skip(self, worker_id), fields(worker_id = %worker_id))]
    async fn _get_latest_heartbeat(&self, worker_id: &Uuid) -> Result<SystemTime, sqlx::Error> {
        self.get_latest_heartbeat(&self.core.pool, worker_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repo::{PgRepositoryCore, PgTaskKindRepository, TaskKindRepository},
        testing::test::init_test_logger,
    };
    use sqlx::PgPool;

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    /// Registers a worker and then retrieves it by id
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn register_and_get_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test task".to_string())
            .await
            .unwrap();

        let worker_id = Uuid::new_v4();
        let worker = repo
            .register_worker(
                worker_id,
                "Test Worker".to_string(),
                vec![task_kind.clone()],
            )
            .await
            .unwrap();

        assert_eq!(worker.id, worker_id);
        assert_eq!(worker.name, "Test Worker");
        assert_eq!(worker.task_kind.len(), 1);
        assert_eq!(worker.task_kind[0].id, task_kind.id);
        assert!(worker.active);

        let retrieved = repo._get_worker_by_id(&worker_id).await.unwrap();
        assert_eq!(worker.id, retrieved.id);
        assert_eq!(worker.name, retrieved.name);
        assert_eq!(worker.task_kind, retrieved.task_kind);
    }

    /// Registers two workers and then retrieves all workers
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_all_workers(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test task".to_string())
            .await
            .unwrap();

        let worker1 = repo
            .register_worker(
                Uuid::new_v4(),
                "Worker 1".to_string(),
                vec![task_kind.clone()],
            )
            .await
            .unwrap();

        let worker2 = repo
            .register_worker(
                Uuid::new_v4(),
                "Worker 2".to_string(),
                vec![task_kind.clone()],
            )
            .await
            .unwrap();

        let all_workers = repo._get_all_workers().await.unwrap();
        assert_eq!(all_workers.len(), 2);
        assert!(all_workers.iter().any(|w| w.id == worker1.id));
        assert!(all_workers.iter().any(|w| w.id == worker2.id));
    }

    /// Tests worker update functionality including name changes and task kind modifications
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn update_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        // Create two distinct task kinds
        let coding_task = task_kind_repo
            .get_or_create_task_kind("Coding".to_string())
            .await
            .unwrap();
        let testing_task = task_kind_repo
            .get_or_create_task_kind("Testing".to_string())
            .await
            .unwrap();

        let worker_id = Uuid::new_v4();

        // Initial worker registration
        let initial_worker = repo
            .register_worker(
                worker_id,
                "Developer Bot".to_string(),
                vec![coding_task.clone()],
            )
            .await
            .unwrap();

        assert_eq!(initial_worker.name, "Developer Bot");
        assert_eq!(initial_worker.task_kind.len(), 1);
        assert_eq!(initial_worker.task_kind[0].id, coding_task.id);

        // Update both name and task kinds
        let updated_worker = repo
            .register_worker(
                worker_id,
                "Test Bot".to_string(),
                vec![testing_task.clone()],
            )
            .await
            .unwrap();

        // Verify the updates
        assert_eq!(updated_worker.id, worker_id);
        assert_eq!(updated_worker.name, "Test Bot");
        assert_eq!(updated_worker.task_kind.len(), 1);
        assert_eq!(updated_worker.task_kind[0].id, testing_task.id);

        // Verify by fetching directly
        let fetched_worker = repo._get_worker_by_id(&worker_id).await.unwrap();
        assert_eq!(fetched_worker.name, "Test Bot");
        assert_eq!(fetched_worker.task_kind.len(), 1);
        assert_eq!(fetched_worker.task_kind[0].id, testing_task.id);
    }

    /// Registers a worker and then updates its active status
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn worker_active_status(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test task".to_string())
            .await
            .unwrap();

        let worker = repo
            .register_worker(Uuid::new_v4(), "Test Worker".to_string(), vec![task_kind])
            .await
            .unwrap();
        assert!(worker.active);

        repo.set_worker_active(&worker.id, false).await.unwrap();
        let updated = repo._get_worker_by_id(&worker.id).await.unwrap();
        assert!(!updated.active);
    }

    /// Registers a worker and then records a heartbeat
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn worker_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let task_kind_repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = task_kind_repo
            .get_or_create_task_kind("Test task".to_string())
            .await
            .unwrap();

        let worker = repo
            .register_worker(Uuid::new_v4(), "Test Worker".to_string(), vec![task_kind])
            .await
            .unwrap();

        repo._record_heartbeat(&worker.id).await.unwrap();
        let heartbeat = repo._get_latest_heartbeat(&worker.id).await.unwrap();

        // Heartbeat should be recent
        let now = SystemTime::now();
        assert!(now.duration_since(heartbeat).unwrap().as_secs() < 1);
    }

    /// Attempts to retrieve a nonexistent worker by id (should fail)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_nonexistent_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));
        let result = repo._get_worker_by_id(&Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    /// Attempts to retrieve a nonexistent heartbeat (should fail)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_nonexistent_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));
        let result = repo._get_latest_heartbeat(&Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}
