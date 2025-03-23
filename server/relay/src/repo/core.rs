use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct PgRepositoryCore {
    pub pool: PgPool,
}

impl PgRepositoryCore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
    }
}
