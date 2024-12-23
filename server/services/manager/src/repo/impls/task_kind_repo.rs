use async_trait::async_trait;
use common::models::TaskKind;
use uuid::Uuid;

use crate::repo::{PgRepositoryCore, TaskKindRepository};

#[derive(Clone)]

pub struct PgTaskKindRepository {
    core: PgRepositoryCore,
}

impl PgTaskKindRepository {
    pub fn new(core: PgRepositoryCore) -> Self {
        Self { core }
    }
}

#[async_trait]
impl TaskKindRepository for PgTaskKindRepository {
    async fn get_or_create_task_kind(&self, name: String) -> Result<TaskKind, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO task_kinds (id, name)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET name = $2
            RETURNING id, name
            "#,
            Uuid::new_v4(),
            name,
        )
        .fetch_one(&self.core.pool)
        .await?;

        Ok(TaskKind {
            id: row.id,
            name: row.name,
        })
    }

    async fn _get_all_task_kinds(&self) -> Result<Vec<TaskKind>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name FROM task_kinds
            "#
        )
        .fetch_all(&self.core.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| TaskKind {
                id: row.id,
                name: row.name,
            })
            .collect())
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
    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn create_new_task_kind(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));
        let name = "Test Task".to_string();

        let task_kind = repo.get_or_create_task_kind(name.clone()).await.unwrap();
        assert_eq!(task_kind.name, name);
    }

    /// Verifies that getting an existing task kind returns the same ID
    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn get_existing_task_kind(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));
        let name = "Test Task".to_string();

        let first = repo.get_or_create_task_kind(name.clone()).await.unwrap();
        let second = repo.get_or_create_task_kind(name).await.unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(first.name, second.name);
    }

    /// Verifies that get_all_task_kinds returns all created task kinds
    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn get_all_task_kinds_test(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let kind1 = repo
            .get_or_create_task_kind("Task 1".to_string())
            .await
            .unwrap();
        let kind2 = repo
            .get_or_create_task_kind("Task 2".to_string())
            .await
            .unwrap();

        let all_kinds = repo._get_all_task_kinds().await.unwrap();

        assert_eq!(all_kinds.len(), 2);
        assert!(all_kinds.iter().any(|k| k.id == kind1.id));
        assert!(all_kinds.iter().any(|k| k.id == kind2.id));
    }
}
