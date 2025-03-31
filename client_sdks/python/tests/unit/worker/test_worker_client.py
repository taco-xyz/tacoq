# pyright: reportPrivateUsage=false
"""Tests for the WorkerApplication functionality.

These tests verify that the worker application can correctly register task handlers,
execute tasks, and manage its lifecycle.
"""

import datetime
import json
from unittest import mock
from uuid import uuid4

import pytest
from aio_pika.abc import AbstractIncomingMessage
from tacoq.core.encoding.pydantic import PydanticDecoder
from tacoq.core.infra.broker import BrokerConfig, WorkerBrokerClient
from tacoq.core.models import (
    TaskAssignmentUpdate,
)
from tacoq.worker import WorkerApplication, WorkerApplicationConfig

from tests.conftest import (
    TestInputPydanticModel,
    TestOutputPydanticModel,
)


# =========================================
# Fixtures
# =========================================


@pytest.fixture
def worker_app():
    """Creates a worker app with mocked dependencies."""

    config = WorkerApplicationConfig(
        name="test_worker",
        kind="test_kind",
        broker_config=BrokerConfig(
            url="amqp://user:password@localhost:5672",
        ),
        broker_prefetch_count=10,
    )
    return WorkerApplication(config=config)


@pytest.fixture
def sample_task_assignment():
    """Creates a sample task assignment for testing."""
    return TaskAssignmentUpdate(
        id=uuid4(),
        task_kind="test_task",
        worker_kind="test_kind",
        input_data=json.dumps({"value": 5}).encode("utf-8"),
        priority=0,
        ttl_duration=60 * 60 * 24 * 7,
        created_at=datetime.datetime.now(),
        otel_ctx_carrier={},
    )


# =========================================
# Task Registration Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_register_single_task(worker_app: WorkerApplication):
    """Test registering a single task handler."""

    async def task_handler(
        input_data: TestInputPydanticModel,
    ) -> TestOutputPydanticModel:
        return TestOutputPydanticModel(value=input_data.value * 2)

    worker_app.register_task(
        "test_task",
        task_handler,
        PydanticDecoder(TestInputPydanticModel),
    )

    assert "test_task" in worker_app._registered_tasks
    assert worker_app._registered_tasks["test_task"].task_function == task_handler


@pytest.mark.unit
@pytest.mark.asyncio
async def test_task_decorator_registration(worker_app: WorkerApplication):
    """Test registering tasks using the decorator."""

    @worker_app.task(
        "decorated_task",
        PydanticDecoder(TestInputPydanticModel),
    )
    async def task_handler(
        input_data: TestInputPydanticModel,
    ) -> TestOutputPydanticModel:
        return TestOutputPydanticModel(value=input_data.value * 2)

    assert "decorated_task" in worker_app._registered_tasks
    assert worker_app._registered_tasks["decorated_task"].task_function == task_handler


@pytest.mark.unit
@pytest.mark.asyncio
async def test_reregister_task(worker_app: WorkerApplication):
    """Test re-registering a task (should overwrite)."""

    async def task1(input_data: TestInputPydanticModel) -> TestOutputPydanticModel:
        return TestOutputPydanticModel(value=input_data.value * 2)

    async def task2(input_data: TestInputPydanticModel) -> TestOutputPydanticModel:
        return TestOutputPydanticModel(value=input_data.value * 2)

    worker_app.register_task(
        "same_kind", task1, PydanticDecoder(TestInputPydanticModel)
    )
    worker_app.register_task(
        "same_kind", task2, PydanticDecoder(TestInputPydanticModel)
    )

    assert worker_app._registered_tasks["same_kind"].task_function == task2


# =========================================
# Task Execution Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_execute_registered_task(
    worker_app: WorkerApplication, sample_task_assignment: TaskAssignmentUpdate
):
    """Test executing a registered task successfully."""
    executed = False
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)

    async def task_handler(
        input_data: TestInputPydanticModel,
    ) -> TestOutputPydanticModel:
        nonlocal executed
        executed = True
        assert input_data == sample_task_assignment.input_data

        return TestOutputPydanticModel(value=input_data.value * 2)

    worker_app.register_task(
        sample_task_assignment.task_kind,
        task_handler,
        input_decoder=PydanticDecoder(TestInputPydanticModel),
    )

    await worker_app._execute_task_assignment(
        sample_task_assignment,
        mock.create_autospec(AbstractIncomingMessage, instance=True),
    )
    assert executed


@pytest.mark.unit
@pytest.mark.asyncio
async def test_execute_unregistered_task(
    worker_app: WorkerApplication, sample_task_assignment: TaskAssignmentUpdate
):
    """Test executing an unregistered task."""
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)

    await worker_app._execute_task_assignment(
        sample_task_assignment,
        mock.create_autospec(AbstractIncomingMessage, instance=True),
    )

    # Verify that the task wasn't published
    worker_app._broker_client.publish_task_completed.assert_not_called()  # type: ignore


@pytest.mark.unit
@pytest.mark.asyncio
async def test_execute_task_with_error(
    worker_app: WorkerApplication, sample_task_assignment: TaskAssignmentUpdate
):
    """Test executing a task that raises an exception."""
    worker_app._broker_client = mock.create_autospec(WorkerBrokerClient, instance=True)

    async def failing_task(_: TestInputPydanticModel) -> TestOutputPydanticModel:
        raise ValueError("Task failed")

    worker_app.register_task(
        sample_task_assignment.task_kind,
        failing_task,
        PydanticDecoder(TestInputPydanticModel),
    )
    await worker_app._execute_task_assignment(
        sample_task_assignment,
        mock.create_autospec(AbstractIncomingMessage, instance=True),
    )
    # TODO: Add assertions for error handling once implemented


# =========================================
# Worker Lifecycle Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_broker_not_initialized(worker_app: WorkerApplication):
    """Test error when broker client is not initialized."""
    worker_app._broker_client = None
    with pytest.raises(RuntimeError, match="Broker client not initialized"):
        await worker_app._listen()
