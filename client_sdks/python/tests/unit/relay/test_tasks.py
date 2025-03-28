"""Tests for the task management functionality of the ManagerClient.

These tests verify that the client can correctly retrieve tasks from the manager
and handle various response scenarios.
"""

import json
from datetime import datetime
from uuid import UUID

import pytest
from aiohttp import ClientResponseError
from aioresponses import aioresponses
from tacoq.relay import RelayClient
from tacoq.core.models import Task, TaskStatus

# =========================================
# Task Retrieval Tests
# =========================================


@pytest.mark.asyncio
@pytest.mark.skip(
    reason="This test doesn't work because I can't make the payload into bytes because aioresponses was made by dumbasses. Re-enable when we use niquests!"
)
async def test_get_task_success(mock_relay_client: RelayClient):
    """Test successful retrieval of a task from the manager."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")
    task = Task(
        id=task_id,
        task_kind="test_kind",
        worker_kind="test_worker_kind",
        created_at=datetime.now(),
        input_data=json.dumps({"foo": "bar"}).encode("utf-8"),
        priority=5,
        output_data=None,
        is_error=0,
        started_at=None,
        completed_at=None,
        executed_by=None,
        otel_ctx_carrier={},
    )

    with aioresponses() as m:
        m.get(  # type: ignore
            f"http://test/tasks/{task_id}",
            payload=task.avro_bytes,
            status=200,
        )
        task = await mock_relay_client.get_task(task_id)
        assert isinstance(task, Task)
        assert task.id == task_id
        assert task.task_kind == "test_kind"
        assert task.status == TaskStatus.PENDING


@pytest.mark.asyncio
async def test_get_task_not_found(mock_relay_client: RelayClient):
    """Test behavior when requesting a non-existent task."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")

    with aioresponses() as m:
        m.get(  # type: ignore
            f"http://test/tasks/{task_id}",
            status=404,
            body=b"Task not found",
            repeat=True,
        )
        response = await mock_relay_client.get_task(task_id)
        assert response is None


@pytest.mark.asyncio
async def test_get_task_server_error(mock_relay_client: RelayClient):
    """Test behavior when the server returns an error response."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")

    with aioresponses() as m:
        m.get(  # type: ignore
            f"http://test/tasks/{task_id}",
            status=500,
            body=b"Internal server error",
            repeat=True,
        )
        with pytest.raises(ClientResponseError) as exc_info:
            await mock_relay_client.get_task(task_id)
        assert exc_info.value.status == 500
