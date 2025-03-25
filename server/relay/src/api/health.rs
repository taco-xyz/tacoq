use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::get, Router};
use tracing::{debug, error, info, instrument};

use crate::lifecycle::RESTServer;

pub fn routes() -> Router<RESTServer> {
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
async fn health(State(state): State<RESTServer>) -> Result<String, (StatusCode, String)> {
    info!("Health check requested");

    let (is_healthy, reports) = state.health_probe.check_health().await;

    if !is_healthy {
        let error_message = reports
            .iter()
            .filter(|report| !report.is_healthy)
            .map(|report| format!("{}: {}", report.component, report.message))
            .collect::<Vec<_>>()
            .join("\n");

        error!("Health check failed: {}", error_message);
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{}\n", error_message),
        ));
    }

    debug!("Health check successful");
    Ok("Service is healthy\n".to_string())
}
