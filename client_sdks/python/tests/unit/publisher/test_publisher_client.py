# pyright: reportPrivateUsage=false
"""Tests for the PublisherClient functionality.

These tests verify that the publisher client can correctly publish tasks
to the broker and retrieve task information from the manager.
"""

from unittest import mock
from uuid import uuid4

import pytest
from tacoq.core.encoding.pydantic import (
    PydanticDecoder,
    PydanticEncoder,
)
from tacoq.core.infra.broker import PublisherBrokerClient
from tacoq.publisher import PublisherClient
from tests.conftest import TestInputPydanticModel

# =========================================
# Task Publishing Tests
# =========================================


@pytest.mark.unit
@pytest.mark.asyncio
async def test_publish_task_success(publisher_client: PublisherClient):
    """Test publishing a task successfully."""
    task_kind = "test_task"
    worker_kind = "test_kind"
    priority = 5
    id = uuid4()

    publisher_client._broker_client = mock.create_autospec(
        PublisherBrokerClient, instance=True
    )

    task = await publisher_client.publish_task(
        task_kind=task_kind,
        worker_kind=worker_kind,
        input_data=TestInputPydanticModel(value=5),
        priority=priority,
        task_id=id,
    )

    # Verify task properties
    assert task.task_kind == task_kind
    assert task.worker_kind == worker_kind
    assert task.get_decoded_input_data(
        PydanticDecoder(TestInputPydanticModel)
    ) == TestInputPydanticModel(value=5)
    assert task.input_data == PydanticEncoder().encode(TestInputPydanticModel(value=5))
    assert task.priority == priority
    assert task.id == id

    # Verify broker client calls
    publisher_client._broker_client.publish_task_assignment.assert_called_once()  # type: ignore
