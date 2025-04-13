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
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            sqlx::query("SELECT 1").fetch_one(&self.pool),
        )
        .await
        .map_err(|_| sqlx::Error::PoolTimedOut)?
        .map(|_| ())
    }
}
