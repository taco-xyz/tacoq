# pyright: reportPrivateUsage=false
"""Tests for the PublisherClient functionality.

These tests verify that the publisher client can correctly publish tasks
to the broker and retrieve task information from the manager.
"""

import json
from unittest import mock
from uuid import uuid4

import pytest
from tacoq.core.infra.broker import PublisherBrokerClient
from tacoq.publisher import PublisherClient

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
