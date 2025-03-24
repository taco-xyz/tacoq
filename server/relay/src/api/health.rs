use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::get, Router};
use tracing::{debug, error, info, instrument};

use crate::lifecycle::AppState;

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
    let result = state.repository_core.health_check().await;
    if let Err(e) = result {
        error!(error = %e, "Database health check failed");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Database is unavailable\n".to_string(),
        ));
    }

    // // Check if broker connection is still alive
    // TODO: change the implementation to use an abstracted broker core instead of a rabbitmq channel
    if let Some(broker_core) = &state.broker_core {
        if let Err(e) = broker_core.health_check().await {
            error!(error = %e, "Broker health check failed");
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Broker is unavailable\n".to_string(),
            ));
        }
    }

    debug!("Health check successful");
    Ok("Service is healthy\n".to_string())
}
