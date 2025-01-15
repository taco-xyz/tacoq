use axum::{routing::get, Json, Router};
use utoipa::OpenApi;

use crate::AppState;

#[derive(OpenApi)]
#[openapi(paths(
    openapi,
    crate::api::tasks::get_task_by_id,
    crate::api::tasks::create_task,
    crate::api::tasks::update_task_status,
    crate::api::tasks::update_task_result,
))]
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
