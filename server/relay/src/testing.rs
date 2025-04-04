#[cfg(test)]
pub mod test {
    use axum_test::TestServer;
    use sqlx::PgPool;

    use crate::lifecycle::setup_app;

    pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    /// Initializes a test logger with debug level output that writes to the test writer.
    /// This should be called at the start of test modules to enable logging during tests.
    pub fn init_test_logger() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }

    /// Creates and returns a test server instance with the application router.
    /// This provides a way to make test HTTP requests against the API endpoints.
    pub async fn get_test_server(db_pools: PgPool) -> TestServer {
        let app = setup_app(&db_pools, None).await;
        TestServer::new(app).unwrap()
    }
}
