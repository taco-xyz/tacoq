use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkerKind {
    pub name: String,
    pub routing_key: String,
    pub queue_name: String,
    pub created_at: OffsetDateTime,
}

impl WorkerKind {
    pub fn new(name: &str, routing_key: &str, queue_name: &str) -> Self {
        WorkerKind {
            name: name.to_string(),
            routing_key: routing_key.to_string(),
            queue_name: queue_name.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub async fn save<'e, E>(&self, executor: E) -> Result<WorkerKind, sqlx::Error>
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
        .bind(&self.name)
        .bind(&self.routing_key)
        .bind(&self.queue_name)
        .bind(self.created_at)
        .fetch_one(executor)
        .await
    }

    pub async fn find_by_name<'e, E>(
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
