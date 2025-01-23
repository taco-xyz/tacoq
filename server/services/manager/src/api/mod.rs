use axum::Router;

use crate::AppState;

mod health;
mod openapi_docs;
mod task;
mod worker_kind;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/worker-kind", worker_kind::routes())
        .nest("/api-docs", openapi_docs::routes())
        .nest("/health", health::routes())
        .nest("/tasks", task::routes())
}
