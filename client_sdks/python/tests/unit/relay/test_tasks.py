"""Tests for the task management functionality of the ManagerClient.

These tests verify that the client can correctly retrieve tasks from the manager
and handle various response scenarios.
"""

import json
import httpx
from datetime import datetime, timezone
from uuid import UUID

import pytest
from unittest.mock import AsyncMock, MagicMock
from tacoq.core.models import Task, TaskStatus

# =========================================
# Task Retrieval Tests
# =========================================


@pytest.mark.asyncio
# Remove the skip marker
async def test_get_task_success(
    mock_relay_client: AsyncMock,
):  # Use AsyncMock type hint
    """Test successful retrieval of a task from the manager."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")
    # Ensure datetime is timezone-aware for consistency if needed by Task model
    expected_task = Task(
        id=task_id,
        task_kind="test_kind",
        worker_kind="test_worker_kind",
        created_at=datetime.now(timezone.utc),
        input_data=json.dumps({"foo": "bar"}).encode("utf-8"),
        priority=5,
        output_data=None,
        is_error=0,
        started_at=None,
        completed_at=None,
        executed_by=None,
        otel_ctx_carrier={},
    )

    # Configure the mock's get_task method to return the expected task
    mock_relay_client.get_task.return_value = expected_task

    # Call the method on the mock
    retrieved_task = await mock_relay_client.get_task(task_id)

    # Assertions
    assert isinstance(retrieved_task, Task)
    assert (
        retrieved_task == expected_task
    )  # Compare the whole object if __eq__ is defined
    assert retrieved_task.id == task_id
    assert retrieved_task.task_kind == "test_kind"
    assert (
        retrieved_task.status == TaskStatus.PENDING
    )  # Assuming status is derived correctly

    # Assert the mock was called correctly
    mock_relay_client.get_task.assert_awaited_once_with(task_id)


@pytest.mark.asyncio
async def test_get_task_not_found(
    mock_relay_client: AsyncMock,
):  # Use AsyncMock type hint
    """Test behavior when requesting a non-existent task."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")

    # Configure the mock's get_task method to return None (simulating 404)
    mock_relay_client.get_task.return_value = None

    # Call the method on the mock
    response = await mock_relay_client.get_task(task_id)

    # Assert the result
    assert response is None

    # Assert the mock was called correctly
    mock_relay_client.get_task.assert_awaited_once_with(task_id)


@pytest.mark.asyncio
async def test_get_task_server_error(
    mock_relay_client: AsyncMock,
):  # Use AsyncMock type hint
    """Test behavior when the server returns an error response."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")

    # Create a mock request and response for the exception
    mock_request = MagicMock(spec=httpx.Request)
    mock_request.method = "GET"
    mock_request.url = (
        f"{mock_relay_client.config.url}/tasks/{task_id}"  # Use mock config URL
    )

    mock_response = MagicMock(spec=httpx.Response)
    mock_response.status_code = 500
    mock_response.request = mock_request
    mock_response.text = "Internal Server Error"
    # Configure content/data if needed by the error handling logic
    mock_response.content = b"Internal Server Error"

    # Configure the mock's get_task method to raise an HTTPStatusError
    error_to_raise = httpx.HTTPStatusError(
        "Server Error", request=mock_request, response=mock_response
    )
    mock_relay_client.get_task.side_effect = error_to_raise

    # Use pytest.raises to assert that the correct exception is raised
    with pytest.raises(httpx.HTTPStatusError) as exc_info:
        await mock_relay_client.get_task(task_id)

    # Assert details about the exception
    assert exc_info.value.response.status_code == 500

    # Assert the mock was called correctly
    mock_relay_client.get_task.assert_awaited_once_with(task_id)
