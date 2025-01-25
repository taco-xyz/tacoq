use async_trait::async_trait;
use common::models::{Worker, WorkerHeartbeat};
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

    pub async fn save_worker<'e, E>(&self, executor: E, w: &Worker) -> Result<Worker, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO workers (id, name, worker_kind_name, registered_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE 
            SET name = $2,
                worker_kind_name = $3
            RETURNING *
            "#,
        )
        .bind(w.id)
        .bind(&w.name)
        .bind(&w.worker_kind_name)
        .bind(w.registered_at)
        .fetch_one(executor)
        .await
    }

    pub async fn find_worker_by_id<'e, E>(
        &self,
        executor: E,
        id: &Uuid,
    ) -> Result<Worker, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM workers WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    pub async fn find_all_workers<'e, E>(&self, executor: E) -> Result<Vec<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM workers")
            .fetch_all(executor)
            .await
    }

    pub async fn save_heartbeat<'e, E>(
        &self,
        executor: E,
        whb: &WorkerHeartbeat,
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
        .bind(whb.worker_id)
        .bind(whb.heartbeat_time)
        .bind(whb.created_at)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn get_latest_heartbeat<'e, E>(
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
        let worker = Worker::new(name, worker_kind_name);
        self.save_worker(&self.core.pool, &worker).await
    }

    #[instrument(skip(self, id), fields(id = %id))]
    async fn _get_worker_by_id(&self, id: &Uuid) -> Result<Worker, sqlx::Error> {
        self.find_worker_by_id(&self.core.pool, id).await
    }

    #[instrument(skip(self))]
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error> {
        self.find_all_workers(&self.core.pool).await
    }

    #[instrument(skip(self, worker_id), fields(worker_id = %worker_id))]
    async fn _record_heartbeat(&self, worker_id: &Uuid) -> Result<(), sqlx::Error> {
        let heartbeat = WorkerHeartbeat::new(*worker_id);
        self.save_heartbeat(&self.core.pool, &heartbeat).await
    }

    #[instrument(skip(self, worker_id), fields(worker_id = %worker_id))]
    async fn _get_latest_heartbeat(&self, worker_id: &Uuid) -> Result<SystemTime, sqlx::Error> {
        let heartbeat = self
            .get_latest_heartbeat(&self.core.pool, worker_id)
            .await?;
        Ok(heartbeat.heartbeat_time.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repo::impls::worker_kind_repo::PgWorkerKindRepository,
        repo::{PgRepositoryCore, WorkerKindRepository},
        testing::test::init_test_logger,
    };
    use common::models::WorkerKind;
    use sqlx::PgPool;
    use std::time::Duration;

    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    async fn setup_test_worker_kind(pool: &PgPool, name: &str) -> WorkerKind {
        let repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool.clone()));
        repo.get_or_create_worker_kind(name, &format!("{}.route", name), &format!("{}_queue", name))
            .await
            .unwrap()
    }

    /// Tests registering workers with different worker kinds
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn worker_kinds(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));

        // Create worker kinds first
        let coding_kind = setup_test_worker_kind(&pool, "coding.worker").await;
        let testing_kind = setup_test_worker_kind(&pool, "testing.worker").await;

        // Register workers with different kinds
        let worker1 = repo
            .register_worker("Worker 1", &coding_kind.name)
            .await
            .unwrap();
        let worker2 = repo
            .register_worker("Worker 2", &testing_kind.name)
            .await
            .unwrap();

        assert_eq!(worker1.worker_kind_name, "coding.worker");
        assert_eq!(worker2.worker_kind_name, "testing.worker");

        // Verify worker kinds are preserved when fetching
        let fetched1 = repo._get_worker_by_id(&worker1.id).await.unwrap();
        let fetched2 = repo._get_worker_by_id(&worker2.id).await.unwrap();

        assert_eq!(fetched1.worker_kind_name, "coding.worker");
        assert_eq!(fetched2.worker_kind_name, "testing.worker");
    }

    /// Registers a worker and then retrieves it by id
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn register_and_get_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker = repo
            .register_worker("Test Worker", &test_kind.name)
            .await
            .unwrap();

        assert_eq!(worker.name, "Test Worker");
        assert_eq!(worker.worker_kind_name, "test.worker");

        let retrieved = repo._get_worker_by_id(&worker.id).await.unwrap();
        assert_eq!(worker.id, retrieved.id);
        assert_eq!(worker.name, retrieved.name);
        assert_eq!(worker.worker_kind_name, retrieved.worker_kind_name);
    }

    /// Registers two workers and then retrieves all workers
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_all_workers(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker1 = repo
            .register_worker("Worker 1", &test_kind.name)
            .await
            .unwrap();

        let worker2 = repo
            .register_worker("Worker 2", &test_kind.name)
            .await
            .unwrap();

        let all_workers = repo._get_all_workers().await.unwrap();
        assert_eq!(all_workers.len(), 2);
        assert!(all_workers.iter().any(|w| w.id == worker1.id));
        assert!(all_workers.iter().any(|w| w.id == worker2.id));
    }

    /// Tests recording and retrieving worker heartbeats
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn worker_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker = repo
            .register_worker("Test Worker", &test_kind.name)
            .await
            .unwrap();

        // Record initial heartbeat
        repo._record_heartbeat(&worker.id).await.unwrap();
        let first_heartbeat = repo._get_latest_heartbeat(&worker.id).await.unwrap();

        // Wait a bit and record another heartbeat
        tokio::time::sleep(Duration::from_millis(100)).await;
        repo._record_heartbeat(&worker.id).await.unwrap();
        let second_heartbeat = repo._get_latest_heartbeat(&worker.id).await.unwrap();

        // Second heartbeat should be more recent than first
        assert!(second_heartbeat > first_heartbeat);
    }

    /// Tests multiple heartbeats from different workers
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn multiple_worker_heartbeats(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker1 = repo
            .register_worker("Worker 1", &test_kind.name)
            .await
            .unwrap();
        let worker2 = repo
            .register_worker("Worker 2", &test_kind.name)
            .await
            .unwrap();

        // Record heartbeats for both workers
        repo._record_heartbeat(&worker1.id).await.unwrap();
        repo._record_heartbeat(&worker2.id).await.unwrap();

        // Each worker should have its own heartbeat
        let heartbeat1 = repo._get_latest_heartbeat(&worker1.id).await.unwrap();
        let heartbeat2 = repo._get_latest_heartbeat(&worker2.id).await.unwrap();

        // Both heartbeats should be recent
        let now = SystemTime::now();
        assert!(now.duration_since(heartbeat1).unwrap().as_secs() < 1);
        assert!(now.duration_since(heartbeat2).unwrap().as_secs() < 1);
    }

    /// Attempts to retrieve a nonexistent worker by id (should fail)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_nonexistent_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let result = repo._get_worker_by_id(&Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    /// Attempts to retrieve a nonexistent heartbeat (should fail)
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_nonexistent_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let result = repo._get_latest_heartbeat(&Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}
