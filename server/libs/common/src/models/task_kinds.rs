use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

// Task Type

/// A task type is a type of task that can be executed by a worker.
/// It is defined by its name and its input data schema.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskKind {
    pub id: Uuid,
    pub name: String,
    pub worker_kind_name: String,
    pub created_at: OffsetDateTime,
}

impl TaskKind {
    pub fn new(name: &str, worker_kind_name: &str) -> TaskKind {
        TaskKind {
            id: Uuid::new_v4(),
            name: name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub async fn save<'e, E>(&self, executor: E) -> Result<TaskKind, sqlx::Error>
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
            RETURNING *
            "#,
        )
        .bind(self.id)
        .bind(&self.name)
        .bind(&self.worker_kind_name)
        .bind(self.created_at)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_name<'e, E>(
        executor: E,
        name: &str,
        worker_kind_name: &str,
    ) -> Result<Option<TaskKind>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(r#"SELECT * FROM task_kinds WHERE name = $1 AND worker_kind_name = $2"#)
            .bind(name)
            .bind(worker_kind_name)
            .fetch_optional(executor)
            .await
    }

    pub async fn find_all<'e, E>(executor: E) -> Result<Vec<TaskKind>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM task_kinds")
            .fetch_all(executor)
            .await
    }
}
