use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub worker_kind_name: String,
    #[serde(
        serialize_with = "crate::models::serialize_datetime",
        deserialize_with = "crate::models::deserialize_datetime"
    )]
    pub created_at: OffsetDateTime,
}

impl Worker {
    pub fn new(name: &str, worker_kind_name: &str) -> Self {
        Worker {
            id: Uuid::new_v4(),
            name: name.to_string(),
            worker_kind_name: worker_kind_name.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub async fn save<'e, E>(&self, executor: E) -> Result<Worker, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            INSERT INTO workers (id, name, worker_kind_name, created_at)
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

    pub async fn find_by_id<'e, E>(executor: E, id: &Uuid) -> Result<Worker, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM workers WHERE id = $1")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    pub async fn find_all<'e, E>(executor: E) -> Result<Vec<Worker>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as("SELECT * FROM workers")
            .fetch_all(executor)
            .await
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, FromRow)]
pub struct WorkerHeartbeat {
    pub worker_id: Uuid,
    pub heartbeat_time: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl WorkerHeartbeat {
    pub fn new(worker_id: Uuid) -> Self {
        WorkerHeartbeat {
            worker_id,
            heartbeat_time: OffsetDateTime::now_utc(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub async fn save<'e, E>(&self, executor: E) -> Result<(), sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO worker_heartbeats (worker_id, heartbeat_time, created_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(self.worker_id)
        .bind(self.heartbeat_time)
        .bind(self.created_at)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn get_latest<'e, E>(
        executor: E,
        worker_id: &Uuid,
    ) -> Result<WorkerHeartbeat, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as(
            r#"
            SELECT * 
            FROM worker_heartbeats 
            WHERE worker_id = $1 
            ORDER BY heartbeat_time DESC 
            LIMIT 1
            "#,
        )
        .bind(worker_id)
        .fetch_one(executor)
        .await
    }
}
