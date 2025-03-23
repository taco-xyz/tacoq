use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::models::{AvroSerializable, Task};
use crate::AppState;

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
/// Returns a response containing the task if found, either in JSON or Avro format based on Accept header
#[utoipa::path(
    get,
    description = "Get a task by its UUID",
    path = "/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID to get")
    ),
    responses(
        (status = 200, description = "Task found", body = Task, content_type = "application/json"),
        (status = 200, description = "Task found (Avro format)", content_type = "application/avro"),
        (status = 404, description = "Task not found", content_type = "text/plain"),
        (status = 500, description = "Internal server error", content_type = "text/plain")
    ),
    tag = "tasks"
)]
#[instrument(skip(state, headers), fields(task_id = %id))]
async fn get_task_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<TaskResponse, (StatusCode, String)> {
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

    match result {
        Ok(Some(task)) => {
            info!(
                task_id = %id,
                task_kind = %task.clone().task_kind.unwrap_or("None".to_string()),
                "Successfully retrieved task"
            );

            // Determine response format based on Accept header
            let format = determine_response_format(&headers);
            debug!(task_id = %id, format = ?format, "Determined response format");

            Ok(TaskResponse { task, format })
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

/// Determines the response format based on the Accept header
fn determine_response_format(headers: &HeaderMap) -> ResponseFormat {
    // Default to JSON if no Accept header is present
    let accept = match headers.get(header::ACCEPT) {
        Some(value) => match value.to_str() {
            Ok(s) => s,
            Err(_) => return ResponseFormat::Json,
        },
        None => return ResponseFormat::Json,
    };

    // Parse accept header parts
    let mut json_quality = 0.0;
    let mut avro_quality = 0.0;

    // Check for wildcard
    if accept.contains("*/*") {
        json_quality = 1.0;
    }

    // Look for quality values or defaults
    for part in accept.split(',').map(|s| s.trim()) {
        if part.starts_with("application/json") {
            json_quality = extract_quality(part).unwrap_or(1.0);
        } else if part.starts_with("application/avro") {
            avro_quality = extract_quality(part).unwrap_or(1.0);
        }
    }

    // Choose format based on quality values
    if avro_quality > 0.0 && avro_quality >= json_quality {
        ResponseFormat::Avro
    } else if json_quality > 0.0 {
        ResponseFormat::Json
    } else {
        // Default to JSON if no matching media type
        ResponseFormat::Json
    }
}

/// Extracts the quality value (q parameter) from an Accept header part
fn extract_quality(part: &str) -> Option<f32> {
    if let Some(q_idx) = part.find(";q=") {
        let q_value = &part[(q_idx + 3)..];
        if let Some(end_idx) = q_value.find(';') {
            q_value[..end_idx].parse::<f32>().ok()
        } else {
            q_value.parse::<f32>().ok()
        }
    } else {
        None
    }
}

/// Response format enum
#[derive(Debug)]
enum ResponseFormat {
    Json,
    Avro,
}

/// Task response wrapper that handles content negotiation
struct TaskResponse {
    task: Task,
    format: ResponseFormat,
}

impl IntoResponse for TaskResponse {
    fn into_response(self) -> Response {
        match self.format {
            ResponseFormat::Json => {
                // Return JSON response
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json")],
                    axum::Json(self.task),
                )
                    .into_response()
            }
            ResponseFormat::Avro => {
                // Convert task to Avro binary format using the convenience method
                match self.task.try_into_avro_bytes() {
                    Ok(avro_bytes) => (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "application/avro")],
                        avro_bytes,
                    )
                        .into_response(),
                    Err(e) => {
                        error!(
                            task_id = %self.task.id,
                            error = %e,
                            "Failed to convert task to Avro bytes"
                        );
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::{AvroSerializable, Task};
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        repo::{PgRepositoryCore, TaskRepository},
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
        let task_repository = TaskRepository::new(core.clone());

        let test_task = get_test_task();

        task_repository.create_task(&test_task).await.unwrap();

        let response = server.get(&format!("/tasks/{}", test_task.id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "application/json"
        );

        let response_body = response.json::<Task>();
        assert_eq!(response_body.id, test_task.id);
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_get_existing_task_by_id_avro(db_pools: PgPool) {
        let server = get_test_server(db_pools.clone()).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_repository = TaskRepository::new(core.clone());

        let test_task = get_test_task();

        task_repository.create_task(&test_task).await.unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::ACCEPT,
            HeaderValue::from_static("application/avro"),
        );

        let response = server
            .get(&format!("/tasks/{}", test_task.id))
            .add_header(
                axum::http::header::ACCEPT,
                HeaderValue::from_static("application/avro"),
            )
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "application/avro"
        );

        let response_body = response.as_bytes().to_vec();
        let expected_task = Task::try_from_avro_bytes(&response_body).unwrap();
        assert_eq!(expected_task.id, test_task.id);
    }

    #[sqlx::test(migrator = "crate::testing::test::MIGRATOR")]
    async fn test_content_negotiation_quality_values(db_pools: PgPool) {
        let server = get_test_server(db_pools.clone()).await;
        let core = PgRepositoryCore::new(db_pools.clone());
        let task_repository = TaskRepository::new(core.clone());

        let test_task = get_test_task();
        task_repository.create_task(&test_task).await.unwrap();

        // Test with higher quality for JSON
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::ACCEPT,
            HeaderValue::from_static("application/json;q=0.9, application/avro;q=0.8"),
        );

        let response = server
            .get(&format!("/tasks/{}", test_task.id))
            .add_header(
                axum::http::header::ACCEPT,
                HeaderValue::from_static("application/json;q=0.9, application/avro;q=0.8"),
            )
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "application/json"
        );

        // Test with higher quality for Avro
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::ACCEPT,
            HeaderValue::from_static("application/json;q=0.7, application/avro;q=0.8"),
        );

        let response = server
            .get(&format!("/tasks/{}", test_task.id))
            .add_header(
                axum::http::header::ACCEPT,
                HeaderValue::from_static("application/json;q=0.7, application/avro;q=0.8"),
            )
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "application/avro"
        );
    }
}
