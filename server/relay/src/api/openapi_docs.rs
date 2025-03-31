use axum::{routing::get, Json, Router};
use tracing::{debug, instrument};
use utoipa::OpenApi;

use crate::lifecycle::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(openapi, crate::api::task::get_task_by_id),
    components(schemas(crate::models::Task)),
    info(
        title = "TacoQ Relay API",
        version = "0.3.1",
        description = "API for the TacoQ Relay Service. Supports both JSON and Avro formats for task data."
    )
)]
struct ApiDoc;

pub fn routes() -> Router<AppState> {
    debug!("Setting up OpenAPI documentation routes");
    Router::new().route("/openapi.json", get(openapi))
}

#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
#[instrument]
async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    debug!("Generating OpenAPI documentation");
    Json(ApiDoc::openapi())
}
