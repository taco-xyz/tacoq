use async_trait::async_trait;
use common::models::TaskKind;
use sqlx::Executor;
use sqlx::Postgres;
use tracing::instrument;

use crate::repo::{PgRepositoryCore, TaskKindRepository};

#[derive(Clone)]
pub struct PgTaskKindRepository {
    core: PgRepositoryCore,
}

impl PgTaskKindRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }

    async fn save_task_kind<'e, E>(
        &self,
        executor: E,
        task_kind: &TaskKind,
    ) -> Result<TaskKind, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO task_kinds (id, name, worker_kind_name, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE 
            SET name = $2,
                worker_kind_name = $3
            RETURNING id, name, worker_kind_name, created_at
            "#,
        )
        .bind(task_kind.id)
        .bind(&task_kind.name)
        .bind(&task_kind.worker_kind_name)
        .bind(task_kind.created_at)
        .fetch_one(executor)
        .await
    }

    async fn find_task_kind_by_name<'e, E>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<TaskKind, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT id, name, worker_kind_name, created_at 
            FROM task_kinds 
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_one(executor)
        .await
    }

    async fn find_all_task_kinds<'e, E>(&self, executor: E) -> Result<Vec<TaskKind>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT id, name, worker_kind_name, created_at FROM task_kinds
            "#,
        )
        .fetch_all(executor)
        .await
    }
}

#[async_trait]
impl TaskKindRepository for PgTaskKindRepository {
    #[instrument(skip(self, name), fields(name = %name))]
    async fn get_or_create_task_kind(&self, name: &str) -> Result<TaskKind, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let result = self.find_task_kind_by_name(&mut *tx, name).await;
        let task_kind = match result {
            Ok(tk) => tk,
            Err(sqlx::Error::RowNotFound) => {
                let task_kind = TaskKind::new(name.to_string(), "default".to_string());
                self.save_task_kind(&mut *tx, &task_kind).await?
            }
            Err(e) => return Err(e),
        };

        tx.commit().await?;
        Ok(task_kind)
    }

    #[instrument(skip(self))]
    async fn _get_all_task_kinds(&self) -> Result<Vec<TaskKind>, sqlx::Error> {
        self.find_all_task_kinds(&self.core.pool).await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::testing::test::init_test_logger;

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    /// Creates a new task kind and verifies it's created correctly
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn create_new_task_kind(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));
        let name = "Test Task".to_string();

        let task_kind = repo.get_or_create_task_kind(&name).await.unwrap();
        assert_eq!(task_kind.name, name);
    }

    /// Verifies that getting an existing task kind returns the same ID
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_existing_task_kind(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));
        let name = "Test Task".to_string();

        let first = repo.get_or_create_task_kind(&name).await.unwrap();
        let second = repo.get_or_create_task_kind(&name).await.unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(first.name, second.name);
    }

    /// Verifies that get_all_task_kinds returns all created task kinds
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_all_task_kinds_test(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let kind1 = repo.get_or_create_task_kind("Task 1").await.unwrap();
        let kind2 = repo.get_or_create_task_kind("Task 2").await.unwrap();

        let all_kinds = repo._get_all_task_kinds().await.unwrap();

        assert_eq!(all_kinds.len(), 2);
        assert!(all_kinds.iter().any(|k| k.id == kind1.id));
        assert!(all_kinds.iter().any(|k| k.id == kind2.id));
    }
}
