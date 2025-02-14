from asyncio import sleep, get_event_loop
import pytest
from uuid import uuid4
import json

from models.task import TaskInput, TaskOutput, TaskStatus
from publisher import PublisherClient
from worker import WorkerApplication
from worker.config import WorkerApplicationConfig
from broker.config import BrokerConfig
from manager.config import ManagerConfig

WORKER_NAME = "test_worker"
WORKER_KIND = "test_worker_kind"

DELAYED_TASK = "delayed_task"
FAILING_TASK = "failing_task"


class WorkerContext:
    """Context manager for running a worker in a separate process."""

    _worker_app: WorkerApplication = None  # type: ignore

    def __init__(self):
        async def delayed_task(input_data: TaskInput) -> TaskOutput:
            await sleep(2)
            return {"message": "Task completed", "input": input_data}

        async def failing_task(_: TaskInput) -> TaskOutput:
            raise ValueError("Task failed intentionally")

        # Create and configure worker
        self._worker_app = WorkerApplication(
            config=WorkerApplicationConfig(
                name=WORKER_NAME,
                kind=WORKER_KIND,
                manager_config=ManagerConfig(url="http://localhost:3000"),
                broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
            )
        )

        # Register appropriate task handler
        self._worker_app.register_task(DELAYED_TASK, delayed_task)
        self._worker_app.register_task(FAILING_TASK, failing_task)

    def __enter__(self):
        # Run worker in background
        loop = get_event_loop()
        loop.create_task(self._worker_app.entrypoint())
        return self._worker_app

    def __exit__(self, exc_type, exc_val, exc_tb):  # type: ignore
        self._worker_app.shutdown()


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_delayed_task_e2e():
    """Test a task that takes 2 seconds to complete.
    This test verifies the full lifecycle of a task:
    1. Task submission
    2. Immediate task status check (should be pending)
    3. Wait for completion
    4. Final task status check (should be completed)
    """
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    # Start worker in background
    with WorkerContext():
        task = await publisher.publish_task(
            task_kind=DELAYED_TASK,
            worker_kind=WORKER_KIND,
            input_data=json.dumps({"test": "data"}),
        )

        await sleep(1)

        # Check immediate status
        task_status = await publisher.get_task(task.id)
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.PENDING
        assert task_status.result is None

        # Wait and check final status
        await sleep(10)  # Wait for task completion + buffer
        task_status = await publisher.get_task(task.id)
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.COMPLETED
        assert task_status.is_error == 0
        assert task_status.result is not None
        assert task_status.result.data["message"] == "Task completed"
        assert task_status.result.data["input"] == {"test": "data"}


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_error_task_e2e():
    """Test a task that fails immediately.
    This test verifies error handling:
    1. Task submission
    2. Task execution (fails)
    3. Task status check (should be failed)
    """
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    # Start worker in background
    with WorkerContext():
        # Submit task
        task = await publisher.publish_task(
            task_kind=FAILING_TASK,
            worker_kind=WORKER_KIND,
            input_data={},
        )

        # Wait a bit for task to be processed
        await sleep(0.5)

        # Check status
        task_status = await publisher.get_task(task.id)
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.COMPLETED
        assert task_status.is_error is True
        assert task_status.result is not None
        assert "Task failed intentionally" in str(task_status.result.data["error"])


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_task_not_found():
    """Test that requesting a non-existent task returns None"""
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    task_status = await publisher.get_task(uuid4())
    assert task_status is None
