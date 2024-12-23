use sqlx::PgPool;

#[derive(Clone)]
pub struct PgRepositoryCore {
    pub pool: PgPool,
}

impl PgRepositoryCore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
