use axum::Router;
use axum_server::{tls_rustls::RustlsConfig, Handle};
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub struct Server {
    app: Router,
    port: u16,
    shutdown_rx: broadcast::Receiver<()>,

    // TLS options
    cert_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
}

impl Server {
    /// Creates a new server that will listen on the given port
    ///
    /// # Arguments
    ///
    /// * `app` - The axum Router to serve
    /// * `port` - The port to listen on
    /// * `shutdown_rx` - Receiver for shutdown signals
    pub fn new(
        app: Router,
        port: u16,
        shutdown_rx: broadcast::Receiver<()>,
        cert_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
    ) -> Self {
        // TODO: should we warn or error out and exit here?
        let (cert_path, key_path) = match (cert_path, key_path) {
            (Some(cert), Some(key)) => (Some(cert), Some(key)),
            (None, None) => (None, None),
            _ => {
                warn!("TLS requires both certificate and key paths. Falling back to HTTP.");
                (None, None)
            }
        };

        Self {
            app,
            port,
            shutdown_rx,
            cert_path,
            key_path,
        }
    }

    /// Runs the server until a shutdown signal is received
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let mut shutdown_rx = self.shutdown_rx.resubscribe();

        // Check if TLS configuration is provided and valid
        if let (Some(cert_path), Some(key_path)) = (&self.cert_path, &self.key_path) {
            info!(address = %addr, "Starting server with TLS (HTTP/1.1 & HTTP/2)");

            // Configure TLS
            let tls_config = match RustlsConfig::from_pem_file(cert_path, key_path).await {
                Ok(config) => config,
                Err(e) => {
                    error!(cert_path = %cert_path.display(), key_path = %key_path.display(), error = %e, "Failed to load TLS certificates");
                    return Err(Box::new(e));
                }
            };

            let handle = Handle::new();

            let handle_clone = handle.clone();
            tokio::spawn(async move {
                match shutdown_rx.recv().await {
                    Ok(_) => info!("Server shutdown signal received, telling axum-server to stop."),
                    Err(e) => error!(error = %e, "Error receiving shutdown signal"),
                }
                handle_clone.graceful_shutdown(None); // Use Some(Duration) for timeout
            });

            // Run the server with TLS
            info!(address = %addr, "Server listening with TLS enabled");
            axum_server::bind_rustls(addr, tls_config)
                .handle(handle) // Pass the handle for graceful shutdown
                .serve(self.app.clone().into_make_service())
                .await?;

            info!("TLS Server shut down gracefully.");

        // --- Non-TLS (HTTP) Path ---
        } else {
            info!(address = %addr, "Starting server without TLS (HTTP only)");

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            info!(address = %addr, "Server listening");

            // Run the server without TLS
            axum::serve(listener, self.app.clone())
                .with_graceful_shutdown(async move {
                    match shutdown_rx.recv().await {
                        Ok(_) => info!("Server shutdown signal received."),
                        Err(e) => error!(error = %e, "Error receiving shutdown signal"),
                    }
                })
                .await?;

            info!("HTTP Server shut down gracefully.");
        }

        Ok(())
    }
}
