use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::models::Task;
use crate::{repo::TaskRepository, AppState};

pub fn routes() -> Router<AppState> {
    debug!("Setting up task API routes");
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
#[instrument(skip(state), fields(task_id = %id))]
async fn get_task_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, (StatusCode, String)> {
    info!(task_id = %id, "API request: Get task by ID");

    let result: Result<Option<Task>, sqlx::Error> =
        match state.task_repository.get_task_by_id(&id).await {
            Ok(task) => Ok(task),
            Err(e) => {
                error!(
                    task_id = %id,
                    error = %e,
                    "Database error while fetching task"
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get task: {}", e),
                ));
            }
        };

    if let Ok(Some(task)) = &result {
        debug!(
            task_id = %id,
            task_kind = %task.task_kind,
            status = %task.status,
            "Task found, checking expiration"
        );

        if task.is_expired() {
            info!(
                task_id = %id,
                ttl_duration = ?task.ttl_duration,
                completed_at = ?task.completed_at,
                "Task is expired, deleting"
            );

            // Delete the task
            if let Err(e) = state.task_repository.delete_task(&task.id).await {
                error!(
                    task_id = %id,
                    error = %e,
                    "Failed to delete expired task"
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to delete expired task: {}", e),
                ));
            }

            debug!(task_id = %id, "Expired task deleted successfully");
            return Err((
                StatusCode::NOT_FOUND,
                format!("Task with ID {} not found", id),
            ));
        }
    }

    match result {
        Ok(Some(task)) => {
            info!(
                task_id = %id,
                task_kind = %task.task_kind,
                status = %task.status,
                "Successfully retrieved task"
            );
            Ok(Json(task))
        }
        Ok(None) => {
            debug!(task_id = %id, "Task not found");
            Err((
                StatusCode::NOT_FOUND,
                format!("Task with ID {} not found", id),
            ))
        }
        Err(_) => unreachable!(), // We handled this above
    }
}

// #[utoipa::path(
//     post,
//     description = "Posts a new task to the consumers",
//     path = "/tasks",
//     request_body = Task,
//     responses(
//         (status = 201, description = "Task created", body = Task, content_type = "application/json"),
//         (status = 500, description = "Internal server error", content_type = "text/plain")
//     ),
//     tag = "tasks"
// )]
// async fn publish_task(
//     State(_state): State<AppState>,
//     Json(task): Json<Task>,
// ) -> Result<Json<Task>, (StatusCode, String)> {
//     Ok(Json(task))
// }

#[cfg(test)]
mod test {
    use crate::models::{Task, TaskStatus};
    use axum::http::StatusCode;
    use chrono::Local;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        repo::{
            PgRepositoryCore, PgTaskRepository, PgWorkerKindRepository, TaskRepository,
            WorkerKindRepository,
        },
        testing::test::{get_test_server, init_test_logger},
    };

    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    fn get_test_task() -> Task {
        Task::new("TaskKindName", "WorkerKindName", 0, 0)
            .with_input_data(vec![1, 2, 3])
            .with_output_data(vec![4, 5, 6])
            .with_error(false)
            .with_status(TaskStatus::Pending)
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_non_existent_task_by_id(db_pools: PgPool) {
        let server = get_test_server(db_pools).await;

        let response = server.get(&format!("/tasks/{}", Uuid::new_v4())).await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_get_existing_task_by_id(db_pools: PgPool) {
        let server = get_test_server(db_pools.clone()).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_instance_repository = PgTaskRepository::new(core.clone());
        let worker_kind_repository = PgWorkerKindRepository::new(core.clone());

        let test_task = get_test_task();

        worker_kind_repository
            .get_or_create_worker_kind(&test_task.worker_kind)
            .await
            .unwrap();

        let task = task_instance_repository
            .update_task(&test_task)
            .await
            .unwrap();

        let response: axum_test::TestResponse = server.get(&format!("/tasks/{}", task.id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_delete_task_with_expired_status(db_pools: PgPool) {
        let server = get_test_server(db_pools.clone()).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_instance_repository = PgTaskRepository::new(core.clone());
        let worker_kind_repository = PgWorkerKindRepository::new(core.clone());

        let mut test_task = get_test_task();
        test_task.status = TaskStatus::Completed;
        test_task.completed_at = Some(Local::now().naive_local() - chrono::Duration::days(1));

        worker_kind_repository
            .get_or_create_worker_kind(&test_task.worker_kind)
            .await
            .unwrap();

        let task = task_instance_repository
            .update_task(&test_task)
            .await
            .unwrap();

        let response: axum_test::TestResponse = server.get(&format!("/tasks/{}", task.id)).await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}
