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
async fn health(State(state): State<AppState>) -> Result<String, (StatusCode, String)> {
    info!("Health check requested");

    // Check if the database connection is still alive
    // TODO: there is perhaps a better way to handle this without using a specific repository
    let result = state.task_repository.health_check().await;
    if let Err(e) = result {
        error!(error = %e, "Database health check failed");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Database is unavailable".to_string(),
        ));
    }

    // Check if broker connection is still alive
    let result = state.task_producer.health_check().await;
    if let Err(e) = result {
        error!(error = %e, "Broker health check failed");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Broker is unavailable".to_string(),
        ));
    }

    debug!("Health check successful");
    Ok("Service is healthy".to_string())
}
