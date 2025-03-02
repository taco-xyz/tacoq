use async_trait::async_trait;
use models::{Worker, WorkerHeartbeat};
use sqlx::{Executor, Postgres};
use std::time::SystemTime;
use tracing::instrument;
use uuid::Uuid;

use crate::repo::{PgRepositoryCore, WorkerRepository};

#[derive(Clone, Debug)]
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
        sqlx::query_as!(
            Worker,
            r#"
            INSERT INTO workers (id, worker_kind_name, registered_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE 
            SET worker_kind_name = $2
            RETURNING *
            "#,
            w.id,
            w.worker_kind_name,
            w.registered_at
        )
        .fetch_one(executor)
        .await
    }

    pub async fn find_worker_by_id<'e, E>(
        &self,
        executor: E,
        id: &Uuid,
    ) -> Result<Option<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(Worker, "SELECT * FROM workers WHERE id = $1", id)
            .fetch_optional(executor)
            .await
    }

    pub async fn find_all_workers<'e, E>(&self, executor: E) -> Result<Vec<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(Worker, "SELECT * FROM workers")
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
        sqlx::query!(
            r#"
            INSERT INTO worker_heartbeats (worker_id, heartbeat_time, created_at)
            VALUES ($1, $2, $3)
            "#,
            whb.worker_id,
            whb.heartbeat_time,
            whb.created_at
        )
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
        sqlx::query_as!(
            WorkerHeartbeat,
            r#"
            SELECT * 
            FROM worker_heartbeats 
            WHERE worker_id = $1 
            ORDER BY heartbeat_time DESC 
            LIMIT 1
            "#,
            worker_id
        )
        .fetch_one(executor)
        .await
    }
}

#[async_trait]
impl WorkerRepository for PgWorkerRepository {
    #[instrument(skip(self, id, worker_kind_name), fields(id = %id, worker_kind_name = %worker_kind_name))]
    async fn update_worker(&self, id: Uuid, worker_kind_name: &str) -> Result<Worker, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let worker = if let Some(worker) = self.find_worker_by_id(&mut *tx, &id).await? {
            worker
        } else {
            let worker = Worker::new(id, worker_kind_name);
            self.save_worker(&mut *tx, &worker).await?
        };

        let heartbeat = WorkerHeartbeat::new(worker.id);
        self.save_heartbeat(&mut *tx, &heartbeat).await?;

        tx.commit().await?;

        Ok(worker)
    }

    #[instrument(skip(self, id), fields(id = %id))]
    async fn _get_worker_by_id(&self, id: &Uuid) -> Result<Option<Worker>, sqlx::Error> {
        self.find_worker_by_id(&self.core.pool, id).await
    }

    #[instrument(skip(self))]
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error> {
        self.find_all_workers(&self.core.pool).await
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
    use models::WorkerKind;
    use sqlx::PgPool;
    use std::time::Duration;

    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    async fn setup_test_worker_kind(pool: &PgPool, name: &str) -> WorkerKind {
        let repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool.clone()));
        repo.get_or_create_worker_kind(
            name, /*, &format!("{}.route", name), &format!("{}_queue", name) */
        )
        .await
        .unwrap()
    }

    /// Tests registering workers with different worker kinds
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn worker_kinds(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));

        // Create worker kinds first
        let coding_kind = setup_test_worker_kind(&pool, "coding.worker").await;
        let testing_kind = setup_test_worker_kind(&pool, "testing.worker").await;

        // Register workers with different kinds
        let worker1 = repo
            .update_worker(Uuid::new_v4(), &coding_kind.name)
            .await
            .unwrap();
        let worker2 = repo
            .update_worker(Uuid::new_v4(), &testing_kind.name)
            .await
            .unwrap();

        assert_eq!(worker1.worker_kind_name, "coding.worker");
        assert_eq!(worker2.worker_kind_name, "testing.worker");

        // Verify worker kinds are preserved when fetching
        let fetched1 = repo._get_worker_by_id(&worker1.id).await.unwrap().unwrap();
        let fetched2 = repo._get_worker_by_id(&worker2.id).await.unwrap().unwrap();

        assert_eq!(fetched1.worker_kind_name, "coding.worker");
        assert_eq!(fetched2.worker_kind_name, "testing.worker");
    }

    /// Registers a worker and then retrieves it by id
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn register_and_get_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;
        let id = Uuid::new_v4();

        let worker = repo.update_worker(id, &test_kind.name).await.unwrap();

        assert_eq!(worker.id, id);
        assert_eq!(worker.worker_kind_name, "test.worker");

        let retrieved = repo._get_worker_by_id(&worker.id).await.unwrap().unwrap();
        assert_eq!(worker.id, retrieved.id);
        assert_eq!(worker.worker_kind_name, retrieved.worker_kind_name);
    }

    /// Registers two workers and then retrieves all workers
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_all_workers(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker1 = repo
            .update_worker(Uuid::new_v4(), &test_kind.name)
            .await
            .unwrap();

        let worker2 = repo
            .update_worker(Uuid::new_v4(), &test_kind.name)
            .await
            .unwrap();

        let all_workers = repo._get_all_workers().await.unwrap();
        assert_eq!(all_workers.len(), 2);
        assert!(all_workers.iter().any(|w| w.id == worker1.id));
        assert!(all_workers.iter().any(|w| w.id == worker2.id));
    }

    /// Tests recording and retrieving worker heartbeats after worker registration
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn worker_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker = repo
            .update_worker(Uuid::new_v4(), &test_kind.name)
            .await
            .unwrap();

        // Get the initial heartbeat that was created during registration
        let initial_heartbeat = repo._get_latest_heartbeat(&worker.id).await.unwrap();

        // Wait a bit and record a new heartbeat
        tokio::time::sleep(Duration::from_millis(100)).await;
        repo.update_worker(worker.id, &test_kind.name)
            .await
            .unwrap();

        let new_heartbeat = repo._get_latest_heartbeat(&worker.id).await.unwrap();

        // New heartbeat should be more recent than the initial one
        assert!(
            new_heartbeat > initial_heartbeat,
            "{}",
            format!("{:?} > {:?}", new_heartbeat, initial_heartbeat)
        );

        // Verify the new heartbeat is recent
        let now = SystemTime::now();
        assert!(now.duration_since(new_heartbeat).unwrap().as_secs() < 1);
    }

    /// Tests multiple heartbeats from different workers
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn multiple_worker_heartbeats(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker1 = repo
            .update_worker(Uuid::new_v4(), &test_kind.name)
            .await
            .unwrap();
        let worker2 = repo
            .update_worker(Uuid::new_v4(), &test_kind.name)
            .await
            .unwrap();

        // Each worker should have its own heartbeat
        let heartbeat1 = repo._get_latest_heartbeat(&worker1.id).await.unwrap();
        let heartbeat2 = repo._get_latest_heartbeat(&worker2.id).await.unwrap();

        // Both heartbeats should be recent
        let now = SystemTime::now();
        assert!(now.duration_since(heartbeat1).unwrap().as_secs() < 1);
        assert!(now.duration_since(heartbeat2).unwrap().as_secs() < 1);
    }

    /// Attempts to retrieve a nonexistent worker by id (should fail)
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_nonexistent_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let result = repo._get_worker_by_id(&Uuid::new_v4()).await;
        assert!(result.is_ok(), "{:?}", result);
        assert!(result.unwrap().is_none());
    }

    /// Attempts to retrieve a nonexistent heartbeat (should fail)
    #[sqlx::test(migrator = "MIGRATOR")]
    async fn get_nonexistent_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let result = repo._get_latest_heartbeat(&Uuid::new_v4()).await;
        assert!(result.is_err(), "{:?}", result);
    }
}
