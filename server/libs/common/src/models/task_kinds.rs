use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
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
    pub fn new(name: String, worker_kind_name: String) -> TaskKind {
        TaskKind {
            id: Uuid::new_v4(),
            name,
            worker_kind_name,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub async fn save(&self, pool: &PgPool) -> Result<TaskKind, sqlx::Error> {
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
        .bind(self.id)
        .bind(&self.name)
        .bind(&self.worker_kind_name)
        .bind(self.created_at)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<TaskKind, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, name, worker_kind_name, created_at 
            FROM task_kinds 
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_one(pool)
        .await
    }

    pub async fn find_all(pool: &PgPool) -> Result<Vec<TaskKind>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, name, worker_kind_name, created_at FROM task_kinds
            "#,
        )
        .fetch_all(pool)
        .await
    }
}
