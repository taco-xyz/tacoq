use async_trait::async_trait;
use common::models::{Worker, WorkerHeartbeat};
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
        worker.save(&self.core.pool).await
    }

    #[instrument(skip(self, id), fields(id = %id))]
    async fn _get_worker_by_id(&self, id: &Uuid) -> Result<Worker, sqlx::Error> {
        Worker::find_by_id(&self.core.pool, id).await
    }

    #[instrument(skip(self))]
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error> {
        Worker::find_all(&self.core.pool).await
    }

    #[instrument(skip(self, worker_id), fields(worker_id = %worker_id))]
    async fn _record_heartbeat(&self, worker_id: &Uuid) -> Result<(), sqlx::Error> {
        let heartbeat = WorkerHeartbeat::new(*worker_id);
        heartbeat.save(&self.core.pool).await
    }

    #[instrument(skip(self, worker_id), fields(worker_id = %worker_id))]
    async fn _get_latest_heartbeat(&self, worker_id: &Uuid) -> Result<SystemTime, sqlx::Error> {
        let heartbeat = WorkerHeartbeat::get_latest(&self.core.pool, worker_id).await?;
        Ok(heartbeat.heartbeat_time.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{repo::PgRepositoryCore, testing::test::init_test_logger};
    use sqlx::PgPool;
    use std::time::Duration;

    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    /// Tests registering workers with different worker kinds
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn worker_kinds(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));

        // Register workers with different kinds
        let worker1 = repo
            .register_worker("Worker 1", "coding.worker")
            .await
            .unwrap();
        let worker2 = repo
            .register_worker("Worker 2", "testing.worker")
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

    /// Tests updating a worker's kind
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn update_worker_kind(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));

        // Register initial worker
        let worker = repo
            .register_worker("Test Worker", "initial.kind")
            .await
            .unwrap();
        assert_eq!(worker.worker_kind_name, "initial.kind");

        // Update worker with new kind
        let updated = repo
            .register_worker("Test Worker", "updated.kind")
            .await
            .unwrap();

        assert_eq!(updated.id, worker.id);
        assert_eq!(updated.worker_kind_name, "updated.kind");

        // Verify update was persisted
        let fetched = repo._get_worker_by_id(&worker.id).await.unwrap();
        assert_eq!(fetched.worker_kind_name, "updated.kind");
    }

    /// Registers a worker and then retrieves it by id
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn register_and_get_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));

        let worker = repo
            .register_worker("Test Worker", "test.worker")
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
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));

        let worker1 = repo
            .register_worker("Worker 1", "test.worker")
            .await
            .unwrap();

        let worker2 = repo
            .register_worker("Worker 2", "test.worker")
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
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));

        let worker = repo
            .register_worker("Test Worker", "test.worker")
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
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool));

        let worker1 = repo
            .register_worker("Worker 1", "test.worker")
            .await
            .unwrap();
        let worker2 = repo
            .register_worker("Worker 2", "test.worker")
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
