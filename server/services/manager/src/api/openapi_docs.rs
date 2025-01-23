use axum::{routing::get, Json, Router};
use utoipa::OpenApi;

use crate::AppState;

#[derive(OpenApi)]
#[openapi(paths(openapi, crate::api::worker_kind::register_worker_kind,))]
struct ApiDoc;

pub fn routes() -> Router<AppState> {
    Router::new().route("/openapi.json", get(openapi))
}

#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
