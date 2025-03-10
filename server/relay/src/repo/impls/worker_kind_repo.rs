use crate::models::WorkerKind;
use async_trait::async_trait;
use sqlx::{Executor, Postgres};
use tracing::{debug, error, info, instrument};

use crate::repo::{PgRepositoryCore, WorkerKindRepository};

#[derive(Clone, Debug)]
pub struct PgWorkerKindRepository {
    core: PgRepositoryCore,
}

impl PgWorkerKindRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }

    pub async fn save<'e, E>(&self, executor: E, w: &WorkerKind) -> Result<WorkerKind, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(
            kind_name = %w.name,
            routing_key = %w.routing_key,
            queue_name = %w.queue_name,
            "Saving worker kind"
        );
        sqlx::query_as!(
            WorkerKind,
            r#"
            INSERT INTO worker_kinds (name, routing_key, queue_name, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (name) DO UPDATE 
            SET routing_key = $2,
                queue_name = $3
            RETURNING *
            "#,
            w.name,
            w.routing_key,
            w.queue_name,
            w.created_at
        )
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_name<'e, E>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<Option<WorkerKind>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        debug!(kind_name = %name, "Finding worker kind by name");
        sqlx::query_as!(
            WorkerKind,
            r#"
            SELECT * FROM worker_kinds WHERE name = $1
            "#,
            name
        )
        .fetch_optional(executor)
        .await
    }
}

#[async_trait]
impl WorkerKindRepository for PgWorkerKindRepository {
    #[instrument(skip(self, name), fields(name = %name))]
    async fn get_or_create_worker_kind(&self, name: &str) -> Result<WorkerKind, sqlx::Error> {
        info!(kind_name = %name, "Getting or creating worker kind");
        let mut tx = match self.core.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!(kind_name = %name, error = %e, "Failed to start transaction");
                return Err(e);
            }
        };

        let worker_kind = if let Some(kind) = self.find_by_name(&mut *tx, name).await? {
            debug!(kind_name = %name, "Worker kind exists, returning");
            kind
        } else {
            info!(kind_name = %name, "Creating new worker kind");
            let worker_kind = WorkerKind::new(name, name, name);
            match self.save(&mut *tx, &worker_kind).await {
                Ok(wk) => wk,
                Err(e) => {
                    error!(kind_name = %name, error = %e, "Failed to save worker kind");
                    return Err(e);
                }
            }
        };

        if let Err(e) = tx.commit().await {
            error!(kind_name = %name, error = %e, "Failed to commit transaction");
            return Err(e);
        }

        info!(
            kind_name = %worker_kind.name,
            routing_key = %worker_kind.routing_key,
            queue_name = %worker_kind.queue_name,
            "Worker kind operation successful"
        );
        Ok(worker_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_worker_kind_operations(pool: PgPool) {
        let repo = PgWorkerKindRepository::new(PgRepositoryCore::new(pool));

        // Test creation
        let worker_kind = repo
            .get_or_create_worker_kind("test" /* , "test.route", "test_queue" */)
            .await
            .unwrap();
        assert_eq!(worker_kind.name, "test");
        assert_eq!(worker_kind.routing_key, "test");
        assert_eq!(worker_kind.queue_name, "test");

        // Test retrieval of existing
        let same_kind = repo
            .get_or_create_worker_kind("test" /* , "test.route", "test_queue" */)
            .await
            .unwrap();
        assert_eq!(worker_kind.name, same_kind.name);
    }
}
