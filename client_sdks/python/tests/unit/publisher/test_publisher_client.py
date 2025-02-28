# pyright: reportPrivateUsage=false

from unittest import mock
from broker.client import PublisherBrokerClient
from manager.client import ManagerClient
import pytest
from uuid import uuid4
import json

from broker.config import BrokerConfig
from manager.config import ManagerConfig
from models.task import Task, TaskStatus
from publisher import PublisherClient


# =========================================
# Fixtures
# =========================================


@pytest.fixture
def publisher_client():
    """Creates a publisher client with mocked dependencies."""

    client = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:8080"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )
    return client


# =========================================
# Task Publishing Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_publish_task_success(publisher_client: PublisherClient):
    """Test publishing a task successfully."""
    task_kind = "test_task"
    worker_kind = "test_kind"
    input_data = {"test": "data"}
    priority = 5
    id = uuid4()

    publisher_client._broker_client = mock.create_autospec(
        PublisherBrokerClient, instance=True
    )

    task = await publisher_client.publish_task(
        task_kind=task_kind,
        worker_kind=worker_kind,
        input_data=json.dumps(input_data),
        priority=priority,
        task_id=id,
    )

    # Verify task properties
    assert task.task_kind == task_kind
    assert task.worker_kind == worker_kind
    assert json.loads(task.input_data) == input_data
    assert task.priority == priority
    assert task.id == id

    # Verify broker client calls
    publisher_client._broker_client.publish_task.assert_called_once_with(  # type: ignore
        task,
    )


# =========================================
# Task Retrieval Tests
# =========================================


@pytest.mark.asyncio
@pytest.mark.unit
async def test_get_task_success(
    publisher_client: PublisherClient,
):
    """Test retrieving a task successfully. Here we mock the manager client because the actual
    task retrieval behaviour is already tested in the manager client tests."""

    task_id = uuid4()
    expected_task = Task(
        id=task_id,
        task_kind="test_task",
        worker_kind="test_kind",
        input_data=json.dumps({"test": "data"}),
        priority=0,
        status=TaskStatus.PENDING,
    )

    publisher_client._manager_client = mock.create_autospec(
        ManagerClient, instance=True
    )
    publisher_client._manager_client.get_task.return_value = expected_task  # type: ignore

    task = await publisher_client.get_task(task_id)
    assert task == expected_task
    publisher_client._manager_client.get_task.assert_called_once_with(  # type: ignore
        task_id,
        override_retry_options=None,
    )
