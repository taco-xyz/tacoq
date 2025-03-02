use axum::http::StatusCode;
use axum::{routing::get, Router};

use crate::AppState;

pub fn routes() -> Router<AppState> {
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
async fn health() -> StatusCode {
    StatusCode::OK
}
