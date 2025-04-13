use axum::Router;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tracing::{error, info};

pub struct Server {
    app: Router,
    port: u16,
    shutdown_rx: broadcast::Receiver<()>,
}

impl Server {
    /// Creates a new server that will listen on the given port
    ///
    /// # Arguments
    ///
    /// * `app` - The axum Router to serve
    /// * `port` - The port to listen on
    /// * `shutdown_rx` - Receiver for shutdown signals
    pub fn new(app: Router, port: u16, shutdown_rx: broadcast::Receiver<()>) -> Self {
        Self {
            app,
            port,
            shutdown_rx,
        }
    }

    /// Runs the server until a shutdown signal is received
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!(address = %addr, "Starting server");

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!(address = %addr, "Server listening");

        let app = self.app.clone();
        let mut shutdown_rx = self.shutdown_rx.resubscribe();

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                if let Err(e) = shutdown_rx.recv().await {
                    error!(error = %e, "Error receiving shutdown signal");
                }
                info!("Server shutdown signal received");
            })
            .await?;

        Ok(())
    }
}
