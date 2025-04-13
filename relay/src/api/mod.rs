use axum::Router;

use crate::lifecycle::AppState;

mod health;
mod openapi_docs;
mod task;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/api-docs", openapi_docs::routes())
        .nest("/health", health::routes())
        .nest("/tasks", task::routes())
}
