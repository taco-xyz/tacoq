use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{delete, post},
    Router,
};
use serde::Deserialize;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use common::models::Worker;

use crate::repo::{TaskKindRepository, WorkerRepository};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(register_worker))
        .route("/:id", delete(unregister_worker))
    // .route("/workers/:id/heartbeat", put(heartbeat))
}

/// Input data for creating a task
#[derive(Debug, Deserialize, ToSchema)]
struct RegisterWorkerInput {
    name: String,
    task_kinds: Vec<String>,
}
/// Register a new worker
///
/// # Arguments
/// * `input` - Input data for registering a worker
///
/// # Returns
/// Returns a JSON response containing the registered worker
#[utoipa::path(
    post,
    description = "Register a new worker",
    path = "/workers",
    request_body = RegisterWorkerInput,
    responses(
        (status = 200, description = "Worker registered", body = Worker, content_type = "application/json"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "workers"
)]
async fn register_worker(
    State(state): State<AppState>,
    Json(input): Json<RegisterWorkerInput>,
) -> Result<(StatusCode, Json<Worker>), (StatusCode, String)> {
    info!("Registering worker with name: {:?}", input.name);

    // Get/create task kinds
    let mut task_kinds = Vec::new();
    for kind_name in input.task_kinds {
        let task_kind = state
            .task_kind_repository
            .get_or_create_task_kind(kind_name)
            .await
            .map_err(|e| {
                error!("Failed to get/create task kind: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get/create task kind: {}", e),
                )
            })?;
        task_kinds.push(task_kind);
    }

    let id = Uuid::new_v4();

    // Register worker in repository
    let worker = state
        .worker_repository
        .register_worker(id, input.name, task_kinds)
        .await
        .map_err(|e| {
            error!("Failed to register worker: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to register worker: {}", e),
            )
        })?;

    // Register worker in broker
    state
        .broker
        .write()
        .await
        .register_worker(worker.clone())
        .await
        .map_err(|e| {
            error!("Failed to register worker in broker: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to register worker in broker: {}", e),
            )
        })?;

    Ok((StatusCode::CREATED, Json(worker)))
}

/// Unregister an existing worker
///
/// # Arguments
/// * `id` - UUID of the worker to unregister
///
/// # Returns
/// Returns a JSON response containing the unregistered worker
#[utoipa::path(
    delete,
    description = "Unregister an existing worker",
    path = "/workers/{id}",
    params(
        ("id" = Uuid, Path, description = "UUID of the worker to unregister")
    ),
    responses(
        (status = 200, description = "Worker unregistered", content_type = "application/json"),
        (status = 404, description = "Worker not found", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "workers"
)]
async fn unregister_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("Unregistering worker with ID: {:?}", id);

    // Set worker as inactive in repository
    state
        .worker_repository
        .set_worker_active(&id, false)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (
                StatusCode::NOT_FOUND,
                format!("Worker with ID {} not found", id),
            ),
            _ => {
                error!("Failed to deactivate worker: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to deactivate worker: {}", e),
                )
            }
        })?;

    // Unregister from broker
    state.broker.write().await.remove_worker(&id).await.map_err(|e| {
        error!("Failed to unregister worker from broker: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to unregister worker from broker: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        repo::{PgRepositoryCore, PgWorkerRepository},
        testing::test::{get_test_server, init_test_logger},
    };
    use common::{brokers::testing::get_mock_broker, TaskKind, Worker};
    use sqlx::PgPool;
    use serde_json::json;

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

    // Test registering a new worker successfully
    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn test_register_worker_success(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools, broker).await;

        let response = server
            .post("/workers")
            .json(&json!({
                "name": "test_worker",
                "task_kinds": ["test_task"]
            }))
            .await;

        assert_eq!(response.status_code(), StatusCode::CREATED);
        let worker: Worker = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(worker.name, "test_worker");
    }

    // Test unregistering an existing worker
    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn test_unregister_worker_success(db_pools: PgPool) {
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

        let response = server.delete(&format!("/workers/{}", test_worker.id)).await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }

    // Test unregistering a non-existent worker
    #[sqlx::test(migrator = "db_common::MIGRATOR")]
    async fn test_unregister_nonexistent_worker(db_pools: PgPool) {
        let broker = get_mock_broker();
        let server = get_test_server(db_pools, broker).await;

        let response = server
            .delete("/workers/123e4567-e89b-12d3-a456-426614174000")
            .await;

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}
