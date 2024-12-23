from worker import WorkerApplication
from manager import ManagerClient
import pytest
from uuid import uuid4
from builtins import anext


# Test Task Definitions. One will fail and one will complete successfully.
async def failing_task(input_data):
    raise Exception("Task failed")


async def successful_task(input_data):
    return input_data


@pytest.mark.asyncio
async def test_worker_startup_and_task_success(
    worker_application: WorkerApplication, manager_client: ManagerClient
):
    """Tests that a worker can start and successfully process a task."""
    TEST_TASK_KIND = str(uuid4())

    # Start worker
    worker_application.register_task(TEST_TASK_KIND, successful_task)
    await worker_application._register_worker()

    try:
        # Create and fetch a task
        input_data = {"test": "data"}
        await manager_client.publish_task(TEST_TASK_KIND, input_data)

        data, task_id, task_kind = await anext(
            worker_application._broker_client.listen()
        )
        assert input_data == data

        # This should execute the task with the given function
        await worker_application._execute_task(task_kind, data, task_id)

        # # Process task successfully
        task = await manager_client.get_task(task_id)
        assert task.has_completed
        assert task.result.data == input_data

    finally:
        await worker_application._unregister_worker()


@pytest.mark.asyncio
async def test_worker_task_failure_handling(
    worker_application: WorkerApplication, manager_client: ManagerClient
):
    """Tests that a worker can properly handle and report task failures."""
    TEST_TASK_KIND = str(uuid4())

    # Start worker
    worker_application.register_task(TEST_TASK_KIND, failing_task)
    await worker_application._register_worker()

    try:
        # Create and fetch a task
        input_data = {"test": "data"}
        await manager_client.publish_task(TEST_TASK_KIND, input_data)

        data, task_id, task_kind = await anext(
            worker_application._broker_client.listen()
        )
        assert input_data == data

        # This should execute the task with the given function
        await worker_application._execute_task(task_kind, data, task_id)

        # Check that the task failed
        task = await manager_client.get_task(task_id)
        assert task.has_failed
        assert "Task failed" in str(task.result.data)

    finally:
        await worker_application._unregister_worker()
