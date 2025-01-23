use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use tracing::{error, info};
use uuid::Uuid;

use common::models::Task;

use crate::{repo::TaskRepository, AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/{id}", get(get_task_by_id))
}

/// Get a task by its UUID
///
/// # Arguments
/// * `id` - UUID of the task to retrieve
///
/// # Returns
/// Returns a JSON response containing the task if found
#[utoipa::path(
    get,
    description = "Get a task by its UUID",
    path = "/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID to get")
    ),
    responses(
        (status = 200, description = "Task found", body = Task, content_type = "application/json"),
        (status = 404, description = "Task not found", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "tasks"
)]
async fn get_task_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, (StatusCode, String)> {
    info!("Getting task by ID: {:?}", id);

    let task = state.task_repository.get_task_by_id(&id).await;

    task.map(Json).map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            info!("Task with ID {:?} not found", id);
            (
                StatusCode::NOT_FOUND,
                format!("Task with ID {} not found", id),
            )
        }
        _ => {
            error!("Error getting task by id: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get task: {}", e),
            )
        }
    })
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use common::brokers::core::MockBrokerProducer;
    use common::models::Task;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tracing::info;

    use crate::{
        repo::{
            PgRepositoryCore, PgTaskKindRepository, PgTaskRepository, TaskKindRepository,
            TaskRepository,
        },
        testing::test::{get_test_server, init_test_logger},
    };

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_non_existent_task_by_id(db_pools: PgPool) {
        let broker = Arc::new(MockBrokerProducer::<Task>::new());
        let server = get_test_server(db_pools, broker).await;

        let response = server
            .get("/tasks/123e4567-e89b-12d3-a456-426614174000")
            .await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_get_existing_task_by_id(db_pools: PgPool) {
        let broker = Arc::new(MockBrokerProducer::<Task>::new());
        let server = get_test_server(db_pools.clone(), broker).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_instance_repository = PgTaskRepository::new(core.clone());
        let task_kind_repository = PgTaskKindRepository::new(core.clone());

        let task_kind = task_kind_repository
            .get_or_create_task_kind("test_task_kind", "test_worker_kind")
            .await
            .unwrap();
        let task = task_instance_repository
            .create_task(task_kind.id, None)
            .await
            .unwrap();

        info!("Task Created: {:?}", task);

        let response: axum_test::TestResponse = server.get(&format!("/tasks/{}", task.id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
