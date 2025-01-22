use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::put,
    Router,
};
use serde::Deserialize;
use serde::Serialize;
use tracing::info;
use utoipa::ToSchema;

use common::models::Worker;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/{kind}", put(register_worker_kind))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct RegisterWorkerKindResponse {
    queue_name: String,
    routing_key: String,
    worker_kind: String,
}

/// Input data for creating a task
#[derive(Debug, Deserialize, ToSchema)]
struct RegisterWorkerKindInput {
    kind: String,
}
/// Register a new worker kind, or do nothing and return the already
/// existing entry if it exists.
///
/// ### Arguments
/// * `state` - The application state
/// * `kind` - The kind of worker to register
///
/// ### Returns
/// Returns a JSON response containing the registered worker kind,
/// including its queue name and associated routing key.
#[utoipa::path(
    put,
    description = "Register a new worker kind",
    path = "/worker-kind/{kind}",
    responses(
        (status = 200, description = "Worker kind registered or retrieved", body = Worker, content_type = "application/json"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "worker-kind"
)]
#[axum::debug_handler]
async fn register_worker_kind(
    State(state): State<AppState>,
    Path(kind): Path<String>,
) -> Result<(StatusCode, Json<RegisterWorkerKindResponse>), (StatusCode, String)> {
    info!("Registering worker kind with name: {:?}", kind);

    // TODO - Attempt database retrieval
    // TODO - Register exchanges and queues
    // TODO - Save in database

    Ok((
        StatusCode::OK,
        Json(RegisterWorkerKindResponse {
            queue_name: format!("worker_kind_{}", kind),
            routing_key: format!("worker_kind_{}", kind),
            worker_kind: kind,
        }),
    ))
}
