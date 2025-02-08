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
    use common::models::Task;
    use sqlx::{types::chrono::Utc, PgPool};
    use tracing::info;
    use uuid::Uuid;

    use crate::{
        repo::{PgRepositoryCore, PgTaskRepository, TaskRepository},
        testing::test::{get_test_server, init_test_logger},
    };

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    fn get_test_task() -> Task {
        Task::new(
            Some(Uuid::new_v4()),
            "TaskKindName",
            "WorkerKindName",
            None,
            None,
            None,
            Utc::now(),
            None,
            None,
            None,
            None,
        )
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_non_existent_task_by_id(db_pools: PgPool) {
        let server = get_test_server(db_pools).await;

        let response = server.get(&format!("/tasks/{}", Uuid::new_v4())).await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "common::MIGRATOR")]
    async fn test_get_existing_task_by_id(db_pools: PgPool) {
        let server = get_test_server(db_pools.clone()).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_instance_repository = PgTaskRepository::new(core.clone());

        let task = task_instance_repository
            .update_task(&get_test_task())
            .await
            .unwrap();

        info!("Task Created: {:?}", task);

        let response: axum_test::TestResponse = server.get(&format!("/tasks/{}", task.id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
