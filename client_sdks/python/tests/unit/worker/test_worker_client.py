# pyright: reportPrivateUsage=false

import asyncio
import datetime
from typing import AsyncGenerator
from unittest import mock
from uuid import uuid4

import pytest
from broker import WorkerBrokerClient
from broker.config import BrokerConfig
from manager.config import ManagerConfig
from models.task import Task, TaskInput, TaskOutput, TaskStatus
from worker import TaskNotRegisteredError, WorkerApplication
from worker.config import WorkerApplicationConfig

# =========================================
# Fixtures
# =========================================


@pytest.fixture
def worker_app():
    """Creates a worker app with mocked dependencies."""

    config = WorkerApplicationConfig(
        name="test_worker",
        kind="test_kind",
        manager_config=ManagerConfig(
            url="http://localhost:8080",
        ),
        broker_config=BrokerConfig(
            url="http://localhost:5672",
        ),
    )
    return WorkerApplication(config=config)


@pytest.fixture
def sample_task():
    """Creates a sample task for testing."""
    return Task(
        id=uuid4(),
        task_kind="test_task",
        worker_kind="test_kind",
        input_data={"value": 5},
        priority=0,
        created_at=datetime.datetime.now(),
        status=TaskStatus.PENDING,
        result=None,
    )


# =========================================
# Task Registration Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_register_single_task(worker_app: WorkerApplication):
    """Test registering a single task handler."""

    async def task_handler(input_data: TaskInput) -> TaskOutput:
        return {"result": input_data["value"] * 2}

    worker_app.register_task("test_task", task_handler)
    assert "test_task" in worker_app._registered_tasks
    assert worker_app._registered_tasks["test_task"] == task_handler


@pytest.mark.unit
@pytest.mark.asyncio
async def test_register_multiple_tasks(worker_app: WorkerApplication):
    """Test registering multiple task handlers."""

    async def task1(input_data: TaskInput) -> TaskOutput:
        return {"result": input_data["value"] * 2}

    async def task2(input_data: TaskInput) -> TaskOutput:
        return {"result": input_data["value"] + 1}

    worker_app.register_task("task1", task1)
    worker_app.register_task("task2", task2)

    assert "task1" in worker_app._registered_tasks
    assert "task2" in worker_app._registered_tasks
    assert worker_app._registered_tasks["task1"] == task1
    assert worker_app._registered_tasks["task2"] == task2


@pytest.mark.unit
@pytest.mark.asyncio
async def test_task_decorator_registration(worker_app: WorkerApplication):
    """Test registering tasks using the decorator."""

    @worker_app.task("decorated_task")
    async def task_handler(input_data: TaskInput) -> TaskOutput:
        return {"result": input_data["value"] * 2}

    assert "decorated_task" in worker_app._registered_tasks
    assert worker_app._registered_tasks["decorated_task"] == task_handler


@pytest.mark.unit
@pytest.mark.asyncio
async def test_reregister_task(worker_app: WorkerApplication):
    """Test re-registering a task (should overwrite)."""

    async def task1(input_data: TaskInput) -> TaskOutput:
        return {"result": 1}

    async def task2(input_data: TaskInput) -> TaskOutput:
        return {"result": 2}

    worker_app.register_task("same_kind", task1)
    worker_app.register_task("same_kind", task2)

    assert worker_app._registered_tasks["same_kind"] == task2


# =========================================
# Task Execution Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_execute_registered_task(
    worker_app: WorkerApplication, sample_task: Task
):
    """Test executing a registered task successfully."""
    executed = False
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)

    async def task_handler(input_data: TaskInput) -> TaskOutput:
        nonlocal executed
        executed = True
        assert input_data == sample_task.input_data
        return {"result": input_data["value"] * 2}

    worker_app.register_task(sample_task.task_kind, task_handler)
    await worker_app._execute_task(sample_task)
    assert executed


@pytest.mark.unit
@pytest.mark.asyncio
async def test_execute_unregistered_task(
    worker_app: WorkerApplication, sample_task: Task
):
    """Test executing an unregistered task."""
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)
    with pytest.raises(TaskNotRegisteredError) as exc_info:
        await worker_app._execute_task(sample_task)
    assert sample_task.task_kind in str(exc_info.value)


@pytest.mark.unit
@pytest.mark.asyncio
async def test_execute_task_with_error(
    worker_app: WorkerApplication, sample_task: Task
):
    """Test executing a task that raises an exception."""
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)

    async def failing_task(_: TaskInput) -> TaskOutput:
        raise ValueError("Task failed")

    worker_app.register_task(sample_task.task_kind, failing_task)
    await worker_app._execute_task(sample_task)
    # TODO: Add assertions for error handling once implemented


# =========================================
# Worker Lifecycle Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_worker_startup(worker_app: WorkerApplication):
    """Test worker startup sequence and the graceful shutdown."""

    # We create a full mock that gracefully shuts down as soon as it is initialized
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)
    if worker_app._broker_client:

        async def mock_listen() -> AsyncGenerator[Task, None]:
            raise asyncio.CancelledError()
            yield None

        worker_app._broker_client.listen = mock_listen  # type: ignore

    await worker_app._listen()


@pytest.mark.unit
@pytest.mark.asyncio
async def test_broker_not_initialized(worker_app: WorkerApplication):
    """Test error when broker client is not initialized."""
    worker_app._broker_client = None
    with pytest.raises(RuntimeError, match="Broker client not initialized"):
        await worker_app._listen()
