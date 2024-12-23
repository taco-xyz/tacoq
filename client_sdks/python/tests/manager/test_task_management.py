from src.manager import ManagerClient
from src.models.task import TaskStatus
import pytest
from uuid import UUID, uuid4


@pytest.mark.asyncio
async def test_task_lifecycle(manager_client: ManagerClient):
    """Tests full task lifecycle - publish, get, update status, update result."""

    # NOTE - We use a random UUID for the task kind to avoid conflicts in parallel tests
    TEST_TASK_KIND = str(uuid4())
    TEST_WORKER_NAME = str(uuid4())

    # Register worker
    worker_id = await manager_client.register_worker(TEST_WORKER_NAME, [TEST_TASK_KIND])

    try:
        # Publish task
        input_data = {"test": "data"}
        task = await manager_client.publish_task(TEST_TASK_KIND, input_data)

        assert task.task_kind == TEST_TASK_KIND
        assert task.input_data == input_data

        # Get task
        retrieved = await manager_client.get_task(task.id)
        assert retrieved.id == task.id
        assert retrieved.task_kind == TEST_TASK_KIND

        # Update status
        await manager_client.update_task_status(task.id, TaskStatus.RUNNING)
        updated = await manager_client.get_task(task.id)
        assert updated.status == TaskStatus.RUNNING

        # Update result
        result_data = {"result": "success"}
        await manager_client.update_task_result(task.id, result_data)
        final = await manager_client.get_task(task.id)
        assert final.result is not None
        assert final.result.data == result_data
        assert not final.result.is_error

    finally:
        await manager_client.unregister_worker(worker_id)


@pytest.mark.asyncio
async def test_task_error_result(manager_client: ManagerClient):
    """Tests submitting error results for tasks."""

    # NOTE - We use a random UUID for the task kind to avoid conflicts in parallel tests
    TEST_TASK_KIND = str(uuid4())
    TEST_WORKER_NAME = str(uuid4())

    worker_id = await manager_client.register_worker(TEST_WORKER_NAME, [TEST_TASK_KIND])

    try:
        task = await manager_client.publish_task(TEST_TASK_KIND)
        error_data = {"error": "test failure"}
        await manager_client.update_task_result(task.id, error_data, is_error=True)

        result = await manager_client.get_task(task.id)
        assert result.result is not None
        assert result.result.is_error
        assert result.result.data == error_data

    finally:
        await manager_client.unregister_worker(worker_id)


@pytest.mark.asyncio
async def test_get_nonexistent_task(manager_client: ManagerClient):
    """Tests getting a task that doesn't exist."""

    fake_id = UUID("00000000-0000-0000-0000-000000000000")
    with pytest.raises(Exception):
        await manager_client.get_task(fake_id)


@pytest.mark.asyncio
async def test_update_nonexistent_task(manager_client: ManagerClient):
    """Tests updating status/result of nonexistent task."""

    fake_id = UUID("00000000-0000-0000-0000-000000000000")

    with pytest.raises(Exception):
        await manager_client.update_task_status(fake_id, TaskStatus.COMPLETED)

    with pytest.raises(Exception):
        await manager_client.update_task_result(fake_id, {"data": "test"})
