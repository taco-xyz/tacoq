use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use common::{models::TaskInstance, TaskStatus};

use crate::{
    repo::{TaskInstanceRepository, TaskKindRepository},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_task_by_id))
        .route("/", post(create_task))
        .route("/:id/status", put(update_task_status))
        .route("/:id/result", put(update_task_result))
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
    path = "/tasks/:id",
    params(
        ("id" = Uuid, Path, description = "Task ID to get")
    ),
    responses(
        (status = 200, description = "Task found", body = TaskInstance, content_type = "application/json"),
        (status = 404, description = "Task not found", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "tasks"
)]
async fn get_task_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskInstance>, (StatusCode, String)> {
    info!("Getting task by ID: {:?}", id);

    let task = state.task_repository.get_task_by_id(&id, true).await;

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

/// Input data for creating a task
#[derive(Debug, Deserialize, ToSchema)]
struct CreateTaskInput {
    task_kind_name: String,
    input_data: Option<serde_json::Value>,
}

/// Create a new task
///
/// # Arguments
/// * `task` - Task object containing name and data
///
/// # Returns
/// Returns a JSON response containing the created task
#[utoipa::path(
    post,
    description = "Create a new task",
    path = "/tasks",
    request_body = CreateTaskInput,
    responses(
        (status = 200, description = "Task created", body = TaskInstance, content_type = "application/json"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "tasks"
)]
async fn create_task(
    State(state): State<AppState>,
    Json(task_input): Json<CreateTaskInput>,
) -> Result<(StatusCode, Json<TaskInstance>), (StatusCode, String)> {
    info!("Creating task with kind: {:?}", task_input.task_kind_name);

    // Get / Create the task kind. If it fails, return an error
    let task_kind = state
        .task_kind_repository
        .get_or_create_task_kind(task_input.task_kind_name)
        .await
        .map_err(|e| {
            error!("Failed to get/create task kind: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get/create task kind: {}", e),
            )
        })?;

    // Create and return the new task instance
    let task = state
        .task_repository
        .create_task(task_kind.id, task_input.input_data)
        .await
        .map_err(|e| {
            error!("Failed to create task: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create task: {}", e),
            )
        })?;

    // Send the task to the worker queue
    // We need to lock the broker because we're
    // sharing it between threads
    let worker_id = state
        .broker
        .write()
        .await
        .publish(&task)
        .await
        .map_err(|e| {
            error!("Failed to publish task to broker: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to publish task to broker: {}", e),
            )
        })?;

    // Assign the task the worker
    state
        .task_repository
        .assign_task_to_worker(&task.id, &worker_id)
        .await
        .map_err(|e| {
            error!("Failed to assign task to worker: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to assign task to worker: {}", e),
            )
        })?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update the status of a task
///
/// # Arguments
/// * `id` - UUID of the task to update
///
/// # Returns
/// Returns a JSON response containing the updated task
#[utoipa::path(
    put,
    description = "Update the status of a task. This should only be used by workers.",
    path = "/tasks/:id/status",
    params(
        ("id" = Uuid, Path, description = "Task ID to update")
    ),
    request_body = String,
    responses(
        (status = 200, description = "Task status updated"),
        (status = 400, description = "Invalid task status", content_type = "text/plain"),
        (status = 404, description = "Task not found", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "tasks"
)]
async fn update_task_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(task_status): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("Updating task status for task: {:?}", id);

    let status = TaskStatus::try_from(task_status.as_str()).map_err(|e| {
        error!("Invalid task status: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid task status: {}. Valid statuses are: pending, queued, running, completed, failed, cancelled", e),
        )
    })?;

    let task = state
        .task_repository
        .get_task_by_id(&id, true)
        .await
        .map_err(|e| {
            error!("Task with id {:?} not found: {:?}", id, e);
            (
                StatusCode::NOT_FOUND,
                format!("Task with id {} not found", id),
            )
        })?;

    state
        .task_repository
        .update_task_status(&task.id, status)
        .await
        .map_err(|e| {
            error!("Failed to update task status: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to update task status: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}

/// Input data for submitting task results
#[derive(Debug, Deserialize, ToSchema)]
struct TaskResultInput {
    data: serde_json::Value,
    is_error: bool,
}

/// Submit results or error for a task
///
/// # Arguments
/// * `id` - UUID of the task to update
/// * `result` - Task result data containing output or error
///
/// # Returns
/// Returns OK if successful
#[utoipa::path(
    put,
    description = "Submit results or error for a task. This should only be used by workers.",
    path = "/tasks/:id/result", 
    params(
        ("id" = Uuid, Path, description = "Task ID to update")
    ),
    request_body = TaskResultInput,
    responses(
        (status = 200, description = "Task result updated"),
        (status = 404, description = "Task not found", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "tasks"
)]
async fn update_task_result(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(result): Json<TaskResultInput>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("Updating task result/error for task: {:?}", id);

    // Get the task and check if it exists
    let task = state
        .task_repository
        .get_task_by_id(&id, true)
        .await
        .map_err(|e| {
            error!("Task not found: {:?}", e);
            (
                StatusCode::NOT_FOUND,
                format!("Task with id {} not found", id),
            )
        })?;

    // If the task is in error state, upload the error, otherwise upload the result
    let result_upload = match result.is_error {
        true => {
            state
                .task_repository
                .upload_task_error(&task.id, &task.assigned_to.unwrap(), result.data)
                .await
        }
        false => {
            state
                .task_repository
                .upload_task_result(&task.id, &task.assigned_to.unwrap(), result.data)
                .await
        }
    };

    // If the upload fails, return an error
    result_upload.map_err(|e| {
        error!("Failed to update task result/error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update task result/error: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use common::{TaskInstance, TaskKind, Worker};
    use common::brokers::testing::get_mock_broker;
    use serde_json::json;
    use sqlx::PgPool;
    use tracing::info;

    use crate::{
        repo::{
            PgRepositoryCore, PgTaskInstanceRepository, PgTaskKindRepository, PgWorkerRepository,
            TaskInstanceRepository, TaskKindRepository, WorkerRepository,
        },
        testing::test::{get_test_server, init_test_logger},
    };


    // This runs before any test in this module
    #[ctor::ctor]
    fn init() {
        init_test_logger();
    }

    fn get_test_worker(task_kind_names: &[&str]) -> Worker {
        Worker::new(
            "test_worker".to_string(),
            task_kind_names
                .iter()
                .map(|name| TaskKind::new(name.to_string()))
                .collect(),
        )
    }

    // Getting Task

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn test_non_existent_task_by_id(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools, broker).await;

        let response = server
            .get("/tasks/123e4567-e89b-12d3-a456-426614174000")
            .await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn test_get_existing_task_by_id(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools.clone(), broker).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_instance_repository = PgTaskInstanceRepository::new(core.clone());
        let task_kind_repository = PgTaskKindRepository::new(core.clone());

        let task_kind = task_kind_repository
            .get_or_create_task_kind("test_task_kind".to_string())
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

    // Creating Task

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn create_task_fails_with_non_existent_worker(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        assert_eq!(
            create_response.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn create_task_fails_with_mismatched_worker_task_kind(db_pools: PgPool) {
        let test_worker = get_test_worker(&["different_task"]);
        let mut broker = get_mock_broker();
        broker
            .register_worker(get_test_worker(&["different_task"]))
            .await
            .unwrap();
        let core = PgRepositoryCore::new(db_pools.clone());
        let worker_repo = PgWorkerRepository::new(core);
        worker_repo
            .register_worker(test_worker.id, test_worker.name, test_worker.task_kind)
            .await
            .unwrap();

        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        assert_eq!(
            create_response.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn create_task_succesfully(db_pools: PgPool) {
        let mut broker = get_mock_broker();
        let test_worker = get_test_worker(&["test_task"]);
        broker.register_worker(test_worker.clone()).await.unwrap();
        let core = PgRepositoryCore::new(db_pools.clone());
        let worker_repo = PgWorkerRepository::new(core);
        worker_repo
            .register_worker(test_worker.id, test_worker.name, test_worker.task_kind)
            .await
            .unwrap();

        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        assert_eq!(create_response.status_code(), StatusCode::CREATED);

        info!("Create Response: {:?}", create_response.text());
        let created_task: TaskInstance = serde_json::from_str(&create_response.text()).unwrap();
        let get_response = server.get(&format!("/tasks/{}", created_task.id)).await;
        assert_eq!(get_response.status_code(), StatusCode::OK);
    }

    // Updating Task Status

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn update_task_status_fails_with_non_existent_task(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools, broker).await;

        let response = server
            .put("/tasks/123e4567-e89b-12d3-a456-426614174000/status")
            .json(&"running")
            .await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn update_task_status_fails_with_invalid_status(db_pools: PgPool) {
        let mut broker = get_mock_broker();
        let test_worker = get_test_worker(&["test_task"]);
        broker.register_worker(test_worker.clone()).await.unwrap();
        let core = PgRepositoryCore::new(db_pools.clone());
        let worker_repo = PgWorkerRepository::new(core);
        worker_repo
            .register_worker(test_worker.id, test_worker.name, test_worker.task_kind)
            .await
            .unwrap();

        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        let created_task: TaskInstance = serde_json::from_str(&create_response.text()).unwrap();

        let response = server
            .put(&format!("/tasks/{}/status", created_task.id))
            .json(&"invalid_status")
            .await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn update_task_status_successfully(db_pools: PgPool) {
        let mut broker = get_mock_broker();
        let test_worker = get_test_worker(&["test_task"]);
        broker.register_worker(test_worker.clone()).await.unwrap();
        let core = PgRepositoryCore::new(db_pools.clone());
        let worker_repo = PgWorkerRepository::new(core);
        worker_repo
            .register_worker(test_worker.id, test_worker.name, test_worker.task_kind)
            .await
            .unwrap();

        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        let created_task: TaskInstance = serde_json::from_str(&create_response.text()).unwrap();

        let response = server
            .put(&format!("/tasks/{}/status", created_task.id))
            .json(&"running")
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    // Updating Task Result

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn update_task_result_fails_with_non_existent_task(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools, broker).await;

        let response = server
            .put("/tasks/123e4567-e89b-12d3-a456-426614174000/result")
            .json(&json!({
                "data": {"result": "test"},
                "is_error": false
            }))
            .await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn update_task_result_successfully(db_pools: PgPool) {
        let mut broker = get_mock_broker();
        let test_worker = get_test_worker(&["test_task"]);
        broker.register_worker(test_worker.clone()).await.unwrap();
        let core = PgRepositoryCore::new(db_pools.clone());
        let worker_repo = PgWorkerRepository::new(core);
        worker_repo
            .register_worker(test_worker.id, test_worker.name, test_worker.task_kind)
            .await
            .unwrap();

        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        let created_task: TaskInstance = serde_json::from_str(&create_response.text()).unwrap();

        let response = server
            .put(&format!("/tasks/{}/result", created_task.id))
            .json(&json!({
                "data": {"result": "test"},
                "is_error": false
            }))
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn update_task_error_successfully(db_pools: PgPool) {
        let mut broker = get_mock_broker();
        let test_worker = get_test_worker(&["test_task"]);
        broker.register_worker(test_worker.clone()).await.unwrap();
        let core = PgRepositoryCore::new(db_pools.clone());
        let worker_repo = PgWorkerRepository::new(core);
        worker_repo
            .register_worker(test_worker.id, test_worker.name, test_worker.task_kind)
            .await
            .unwrap();

        let server = get_test_server(db_pools, broker).await;

        let create_response = server
            .post("/tasks")
            .json(&json!({
                "task_kind_name": "test_task",
                "input_data": null
            }))
            .await;
        let created_task: TaskInstance = serde_json::from_str(&create_response.text()).unwrap();

        let response = server
            .put(&format!("/tasks/{}/result", created_task.id))
            .json(&json!({
                "data": {"error": "test error"},
                "is_error": true
            }))
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }
}
