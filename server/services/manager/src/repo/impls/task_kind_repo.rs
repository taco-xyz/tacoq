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
    async fn get_or_create_task_kind(
        &self,
        name: &str,
        worker_kind_name: &str,
    ) -> Result<TaskKind, sqlx::Error> {
        let mut tx = self.core.pool.begin().await?;

        let task_kind = TaskKind::find_by_name(&mut *tx, name, worker_kind_name)
            .await?
            .unwrap_or_else(|| TaskKind::new(name, worker_kind_name));

        tx.commit().await?;
        Ok(task_kind)
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
        let worker_kind_name = "Test Worker".to_string();

        let task_kind = repo
            .get_or_create_task_kind(&name, &worker_kind_name)
            .await
            .unwrap();
        assert_eq!(task_kind.name, name);
    }

    /// Verifies that getting an existing task kind returns the same ID
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_existing_task_kind(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));
        let name = "Test Task".to_string();
        let worker_kind_name = "Test Worker".to_string();

        let first = repo
            .get_or_create_task_kind(&name, &worker_kind_name)
            .await
            .unwrap();
        let second = repo
            .get_or_create_task_kind(&name, &worker_kind_name)
            .await
            .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(first.name, second.name);
    }

    /// Verifies that get_all_task_kinds returns all created task kinds
    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn get_all_task_kinds_test(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let worker_kind_name = "Test Worker".to_string();

        let kind1 = repo
            .get_or_create_task_kind("Task 1", &worker_kind_name)
            .await
            .unwrap();
        let kind2 = repo
            .get_or_create_task_kind("Task 2", &worker_kind_name)
            .await
            .unwrap();

        let all_kinds = repo._get_all_task_kinds().await.unwrap();

        assert_eq!(all_kinds.len(), 2);
        assert!(all_kinds.iter().any(|k| k.id == kind1.id));
        assert!(all_kinds.iter().any(|k| k.id == kind2.id));
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_task_kind_operations(pool: PgPool) {
        let repo = PgTaskKindRepository::new(PgRepositoryCore::new(pool));

        let task_kind = repo.get_or_create_task_kind("test", "test").await.unwrap();
        assert_eq!(task_kind.name, "test");
        assert_eq!(task_kind.worker_kind_name, "test");

        let same_kind = repo.get_or_create_task_kind("test", "test").await.unwrap();
        assert_eq!(task_kind.id, same_kind.id);
    }
}
