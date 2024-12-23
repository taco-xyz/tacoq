use axum::Router;

use crate::AppState;

mod health;
mod openapi_docs;
mod tasks;
mod workers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/tasks", tasks::routes())
        .nest("/workers", workers::routes())
        .nest("/api-docs", openapi_docs::routes())
        .nest("/health", health::routes())
}
