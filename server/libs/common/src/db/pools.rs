use sqlx::PgPool;
use std::error::Error;

#[derive(Clone)]
pub struct DatabasePools {
    pub reader: PgPool,
    pub writer: PgPool,
}

impl DatabasePools {
    pub async fn new(reader_url: &str, writer_url: &str) -> Result<Self, Box<dyn Error>> {
        let reader = PgPool::connect(reader_url).await?;
        let writer = PgPool::connect(writer_url).await?;

        Ok(Self { reader, writer })
    }
}
