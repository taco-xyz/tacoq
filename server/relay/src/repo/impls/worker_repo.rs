use crate::models::{Worker, WorkerHeartbeat};
use async_trait::async_trait;
use sqlx::{Executor, Postgres};
use std::time::SystemTime;
use tracing::{debug, error, info, instrument};

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
        debug!(worker_id = %w.name, worker_kind = %w.worker_kind_name, "Saving worker");
        sqlx::query_as!(
            Worker,
            r#"
            INSERT INTO workers (name, worker_kind_name, registered_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO UPDATE 
            SET worker_kind_name = $2
            RETURNING *
            "#,
            w.name,
            w.worker_kind_name,
            w.registered_at
        )
        .fetch_one(executor)
        .await
    }

    pub async fn find_worker_by_name<'e, E>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<Option<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(worker_name = %name, "Finding worker by name");
        sqlx::query_as!(Worker, "SELECT * FROM workers WHERE name = $1", name)
            .fetch_optional(executor)
            .await
    }

    pub async fn find_all_workers<'e, E>(&self, executor: E) -> Result<Vec<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!("Finding all workers");
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
        debug!(
            worker_name = %whb.worker_name,
            heartbeat_time = ?whb.heartbeat_time,
            "Saving worker heartbeat"
        );
        sqlx::query!(
            r#"
            INSERT INTO worker_heartbeats (worker_name, heartbeat_time, created_at)
            VALUES ($1, $2, $3)
            "#,
            whb.worker_name,
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
        worker_name: &str,
    ) -> Result<WorkerHeartbeat, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(worker_name = %worker_name, "Getting latest heartbeat");
        sqlx::query_as!(
            WorkerHeartbeat,
            r#"
            SELECT * 
            FROM worker_heartbeats 
            WHERE worker_name = $1 
            ORDER BY heartbeat_time DESC 
            LIMIT 1
            "#,
            worker_name
        )
        .fetch_one(executor)
        .await
    }
}

#[async_trait]
impl WorkerRepository for PgWorkerRepository {
    #[instrument(skip(self, name, worker_kind_name), fields(name = %name, worker_kind_name = %worker_kind_name))]
    async fn update_worker(
        &self,
        name: &str,
        worker_kind_name: &str,
    ) -> Result<Worker, sqlx::Error> {
        info!(worker_name = %name, worker_kind = %worker_kind_name, "Updating worker");
        let mut tx = match self.core.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!(worker_name = %name, error = %e, "Failed to start transaction");
                return Err(e);
            }
        };

        let worker = if let Some(worker) = self.find_worker_by_name(&mut *tx, name).await? {
            debug!(worker_name = %name, "Worker exists, updating");
            worker
        } else {
            info!(worker_name = %name, worker_kind = %worker_kind_name, "Creating new worker");
            let worker = Worker::new(name, worker_kind_name);
            match self.save_worker(&mut *tx, &worker).await {
                Ok(w) => w,
                Err(e) => {
                    error!(worker_name = %name, error = %e, "Failed to save worker");
                    return Err(e);
                }
            }
        };

        let heartbeat = WorkerHeartbeat::new(worker.name.as_str());
        if let Err(e) = self.save_heartbeat(&mut *tx, &heartbeat).await {
            error!(worker_name = %name, error = %e, "Failed to save heartbeat");
            return Err(e);
        }

        if let Err(e) = tx.commit().await {
            error!(worker_name = %name, error = %e, "Failed to commit transaction");
            return Err(e);
        }

        info!(worker_name = %name, worker_kind = %worker_kind_name, "Worker successfully updated");
        Ok(worker)
    }

    #[instrument(skip(self, name))]
    async fn _get_worker_by_name(&self, name: &str) -> Result<Option<Worker>, sqlx::Error> {
        debug!(worker_name = %name, "Getting worker by name");
        let result = self.find_worker_by_name(&self.core.pool, name).await;

        match &result {
            Ok(Some(worker)) => info!(
                worker_name = %name,
                worker_kind = %worker.worker_kind_name,
                "Worker found"
            ),
            Ok(None) => debug!(worker_name = %name, "Worker not found"),
            Err(e) => error!(
                worker_name = %name,
                error = %e,
                "Error fetching worker"
            ),
        }

        result
    }

    #[instrument(skip(self))]
    async fn _get_all_workers(&self) -> Result<Vec<Worker>, sqlx::Error> {
        info!("Getting all workers");
        let result = self.find_all_workers(&self.core.pool).await;

        match &result {
            Ok(workers) => info!(count = workers.len(), "Retrieved all workers"),
            Err(e) => error!(error = %e, "Failed to retrieve all workers"),
        }

        result
    }

    #[instrument(skip(self, worker_name))]
    async fn _get_latest_heartbeat(&self, worker_name: &str) -> Result<SystemTime, sqlx::Error> {
        debug!(worker_name = %worker_name, "Getting latest heartbeat");
        let heartbeat = match self
            .get_latest_heartbeat(&self.core.pool, worker_name)
            .await
        {
            Ok(hb) => {
                debug!(
                    worker_name = %worker_name,
                    heartbeat_time = ?hb.heartbeat_time,
                    "Retrieved heartbeat"
                );
                hb
            }
            Err(e) => {
                error!(worker_name = %worker_name, error = %e, "Failed to retrieve heartbeat");
                return Err(e);
            }
        };

        Ok(heartbeat.heartbeat_time.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WorkerKind;
    use crate::{
        repo::impls::worker_kind_repo::PgWorkerKindRepository,
        repo::{PgRepositoryCore, WorkerKindRepository},
        testing::test::init_test_logger,
    };
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
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn worker_kinds(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));

        // Create worker kinds first
        let coding_kind = setup_test_worker_kind(&pool, "coding.worker").await;
        let testing_kind = setup_test_worker_kind(&pool, "testing.worker").await;

        // Register workers with different kinds
        let worker1 = repo
            .update_worker("test.worker", &coding_kind.name)
            .await
            .unwrap();
        let worker2 = repo
            .update_worker("test.worker", &testing_kind.name)
            .await
            .unwrap();

        assert_eq!(worker1.worker_kind_name, "coding.worker");
        assert_eq!(worker2.worker_kind_name, "testing.worker");

        // Verify worker kinds are preserved when fetching
        let fetched1 = repo
            ._get_worker_by_name(&worker1.name)
            .await
            .unwrap()
            .unwrap();
        let fetched2 = repo
            ._get_worker_by_name(&worker2.name)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched1.worker_kind_name, "coding.worker");
        assert_eq!(fetched2.worker_kind_name, "testing.worker");
    }

    /// Registers a worker and then retrieves it by id
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn register_and_get_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;
        let name = "test.worker";

        let worker = repo.update_worker(name, &test_kind.name).await.unwrap();

        assert_eq!(worker.name, name);
        assert_eq!(worker.worker_kind_name, "test.worker");

        let retrieved = repo
            ._get_worker_by_name(&worker.name)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(worker.name, retrieved.name);
        assert_eq!(worker.worker_kind_name, retrieved.worker_kind_name);
    }

    /// Registers two workers and then retrieves all workers
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn get_all_workers(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker1 = repo
            .update_worker("test.worker", &test_kind.name)
            .await
            .unwrap();

        let worker2 = repo
            .update_worker("test.worker", &test_kind.name)
            .await
            .unwrap();

        let all_workers = repo._get_all_workers().await.unwrap();
        assert_eq!(all_workers.len(), 2);
        assert!(all_workers.iter().any(|w| w.name == worker1.name));
        assert!(all_workers.iter().any(|w| w.name == worker2.name));
    }

    /// Tests recording and retrieving worker heartbeats after worker registration
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn worker_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker = repo
            .update_worker("test.worker", &test_kind.name)
            .await
            .unwrap();

        // Get the initial heartbeat that was created during registration
        let initial_heartbeat = repo._get_latest_heartbeat(&worker.name).await.unwrap();

        // Wait a bit and record a new heartbeat
        tokio::time::sleep(Duration::from_millis(100)).await;
        repo.update_worker(&worker.name, &test_kind.name)
            .await
            .unwrap();

        let new_heartbeat = repo._get_latest_heartbeat(&worker.name).await.unwrap();

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
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn multiple_worker_heartbeats(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let test_kind = setup_test_worker_kind(&pool, "test.worker").await;

        let worker1 = repo
            .update_worker("test.worker", &test_kind.name)
            .await
            .unwrap();
        let worker2 = repo
            .update_worker("test.worker", &test_kind.name)
            .await
            .unwrap();

        // Each worker should have its own heartbeat
        let heartbeat1 = repo._get_latest_heartbeat(&worker1.name).await.unwrap();
        let heartbeat2 = repo._get_latest_heartbeat(&worker2.name).await.unwrap();

        // Both heartbeats should be recent
        let now = SystemTime::now();
        assert!(now.duration_since(heartbeat1).unwrap().as_secs() < 1);
        assert!(now.duration_since(heartbeat2).unwrap().as_secs() < 1);
    }

    /// Attempts to retrieve a nonexistent worker by id (should fail)
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn get_nonexistent_worker(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let result = repo._get_worker_by_name("nonexistent.worker").await;
        assert!(result.is_ok(), "{:?}", result);
        assert!(result.unwrap().is_none());
    }

    /// Attempts to retrieve a nonexistent heartbeat (should fail)
    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn get_nonexistent_heartbeat(pool: PgPool) {
        let repo = PgWorkerRepository::new(PgRepositoryCore::new(pool.clone()));
        let result = repo._get_latest_heartbeat("nonexistent.worker").await;
        assert!(result.is_err(), "{:?}", result);
    }
}
