use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::oneshot;
use tracing::info;

pub struct Server {
    app: Router,
    addr: SocketAddr,
}

impl Server {
    pub fn new(app: Router, port: u16) -> Self {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        Self { app, addr }
    }

    pub async fn run(
        self,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Listening on {}", self.addr);
        let listener = tokio::net::TcpListener::bind(self.addr).await?;

        axum::serve(listener, self.app)
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await?;

        Ok(())
    }
}
