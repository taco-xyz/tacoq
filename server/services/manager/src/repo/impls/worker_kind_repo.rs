use async_trait::async_trait;
use common::models::WorkerKind;
use sqlx::{Executor, Postgres};
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
impl WorkerKindRepository for PgWorkerKindRepository {
    #[instrument(skip(self, name), fields(name = %name))]
    async fn get_or_create_worker_kind(
        &self,
        name: &str,
        exchange: &str,
        queue: &str,
    ) -> Result<WorkerKind, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let worker_kind = self.find_worker_kind_by_name(&mut *tx, name).await?;
        if let Some(worker_kind) = worker_kind {
            return Ok(worker_kind);
        } else {
            let worker_kind = WorkerKind::new(name, exchange, queue);
            let worker_kind = self.save_worker_kind(&mut *tx, &worker_kind).await?;
            tx.commit().await?;
            Ok(worker_kind)
        }
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
