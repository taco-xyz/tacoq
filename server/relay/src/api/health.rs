use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::get, Router};
use tracing::{debug, error, info, instrument};

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
#[instrument(skip(state))]
async fn health(State(state): State<AppState>) -> StatusCode {
    info!("Health check requested");

    // Check if the database connection is still alive
    // TODO: there is perhaps a better way to handle this without using a specific repository
    let result = state.task_repository.health_check().await;
    if let Err(e) = result {
        error!(error = %e, "Database health check failed");
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    // Check if broker connection is still alive
    let result = state.task_producer.health_check().await;
    if let Err(e) = result {
        error!(error = %e, "Broker health check failed");
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    debug!("Health check successful");
    StatusCode::OK
}
