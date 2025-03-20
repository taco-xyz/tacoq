use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::get, Router};
use tracing::{debug, info, instrument};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    debug!("Setting up health API routes");
    Router::new().route("/", get(health))
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service is unhealthy")
    ),
    tag = "health"
)]
#[instrument(skip(_state))]
async fn health(State(_state): State<AppState>) -> StatusCode {
    info!("Health check requested");
    // Here you could add additional health checks for backend services
    debug!("Health check successful");
    StatusCode::OK
}
