use async_trait::async_trait;
use common::models::WorkerKind;
use tracing::instrument;

use crate::repo::{PgRepositoryCore, WorkerKindRepository};

#[derive(Clone)]
pub struct PgWorkerKindRepository {
    core: PgRepositoryCore,
}

impl PgWorkerKindRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }
}

#[async_trait]
impl WorkerKindRepository for PgWorkerKindRepository {
    #[instrument(skip(self, name), fields(name = %name))]
    async fn get_or_create_worker_kind(
        &self,
        name: &str,
        exchange: &str,
        queue: &str,
    ) -> Result<WorkerKind, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let worker_kind = WorkerKind::find_by_name(&mut *tx, name)
            .await?
            .unwrap_or_else(|| WorkerKind::new(name, exchange, queue));

        worker_kind.save(&mut *tx).await?;
        tx.commit().await?;
        Ok(worker_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_worker_kind_operations(pool: PgPool) {
        let repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool));

        // Test creation
        let worker_kind = repo
            .get_or_create_worker_kind("test", "test.route", "test_queue")
            .await
            .unwrap();
        assert_eq!(worker_kind.name, "test");
        assert_eq!(worker_kind.routing_key, "test.route");
        assert_eq!(worker_kind.queue_name, "test_queue");

        // Test retrieval of existing
        let same_kind = repo
            .get_or_create_worker_kind("test", "test.route", "test_queue")
            .await
            .unwrap();
        assert_eq!(worker_kind.name, same_kind.name);
    }
}
