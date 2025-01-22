use async_trait::async_trait;
use common::models::TaskKind;
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
}

#[async_trait]
impl TaskKindRepository for PgTaskKindRepository {
    #[instrument(skip(self, name), fields(name = %name))]
    async fn get_or_create_task_kind(&self, name: &str) -> Result<TaskKind, sqlx::Error> {
        let result = TaskKind::find_by_name(&self.core.pool, name).await;

        // If the task kind doesn't exist, create it
        if let Err(sqlx::Error::RowNotFound) = result {
            let task_kind = TaskKind::new(name.to_string(), "default".to_string());
            task_kind.save(&self.core.pool).await
        } else {
            result
        }
    }

    #[instrument(skip(self))]
    async fn _get_all_task_kinds(&self) -> Result<Vec<TaskKind>, sqlx::Error> {
        TaskKind::find_all(&self.core.pool).await
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
