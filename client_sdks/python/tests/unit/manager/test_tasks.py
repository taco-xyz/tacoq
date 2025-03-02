"""Tests for the task management functionality of the ManagerClient.

These tests verify that the client can correctly retrieve tasks from the manager
and handle various response scenarios.
"""

import pytest
from uuid import UUID
from aiohttp import ClientResponseError
from aioresponses import aioresponses
import json

from manager.client import ManagerClient
from models.task import Task, TaskStatus


# =========================================
# Task Retrieval Tests
# =========================================


@pytest.mark.asyncio
async def test_get_task_success(mock_manager_client: ManagerClient):
    """Test successful retrieval of a task from the manager."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")
    task_data = {
        "id": str(task_id),
        "task_kind": "test_kind",
        "worker_kind": "test_worker_kind",
        "created_at": "2024-01-01T00:00:00Z",
        "input_data": json.dumps({"foo": "bar"}),
        "status": TaskStatus.PENDING.value,  # Use enum value for serialization
        "priority": 5,
        "result": None,
    }

    with aioresponses() as m:
        m.get(  # type: ignore
            f"http://test/tasks/{task_id}",
            payload=task_data,
            status=200,
        )
        task = await mock_manager_client.get_task(task_id)
        assert isinstance(task, Task)
        assert task.id == task_id
        assert task.task_kind == "test_kind"
        assert task.status == TaskStatus.PENDING


@pytest.mark.asyncio
async def test_get_task_not_found(mock_manager_client: ManagerClient):
    """Test behavior when requesting a non-existent task."""
    task_id = UUID("00000000-0000-0000-0000-000000000000")

    with aioresponses() as m:
        m.get(  # type: ignore
            f"http://test/tasks/{task_id}",
            status=404,
            body=b"Task not found",
            repeat=True,
        )
        response = await mock_manager_client.get_task(task_id)
        assert response is None


@pytest.mark.asyncio
async def test_get_task_server_error(mock_manager_client: ManagerClient):
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
            await mock_manager_client.get_task(task_id)
        assert exc_info.value.status == 500
