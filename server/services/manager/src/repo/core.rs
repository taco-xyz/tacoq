use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct PgRepositoryCore {
    pub pool: PgPool,
}

impl PgRepositoryCore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
