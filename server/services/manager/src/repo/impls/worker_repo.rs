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
