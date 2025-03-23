import uuid
from datetime import datetime, timezone

import pytest
from tacoq.core.models.task import Task


@pytest.mark.unit
def test_task_avro_serde():
    task = Task(
        id=uuid.uuid4(),
        task_kind="test_task",
        worker_kind="test_worker",
        created_at=datetime.now(timezone.utc),
        started_at=datetime.now(timezone.utc),
        completed_at=datetime.now(timezone.utc),
        executed_by="test_worker",
        input_data=b"test input",
        output_data=b"test output",
        is_error=0,
        priority=128,
        ttl_duration=3600,
        otel_ctx_carrier={"trace_id": "123"},
    )

    # Convert to Avro bytes
    avro_bytes = task.avro_bytes

    # Convert back from Avro bytes
    deserialized = Task.from_avro_bytes(avro_bytes)

    # Check all fields match
    assert task.id == deserialized.id
    assert task.task_kind == deserialized.task_kind
    assert task.worker_kind == deserialized.worker_kind
    assert task.created_at.timestamp() == deserialized.created_at.timestamp()

    # Handle nullable datetime fields
    if task.started_at is not None and deserialized.started_at is not None:
        assert task.started_at.timestamp() == deserialized.started_at.timestamp()
    else:
        assert task.started_at == deserialized.started_at

    if task.completed_at is not None and deserialized.completed_at is not None:
        assert task.completed_at.timestamp() == deserialized.completed_at.timestamp()
    else:
        assert task.completed_at == deserialized.completed_at

    assert task.executed_by == deserialized.executed_by
    assert task.input_data == deserialized.input_data
    assert task.output_data == deserialized.output_data
    assert task.is_error == deserialized.is_error
    assert task.priority == deserialized.priority
    assert task.ttl_duration == deserialized.ttl_duration
    assert task.otel_ctx_carrier == deserialized.otel_ctx_carrier


@pytest.mark.unit
def test_task_avro_serde_with_nulls():
    """Test Avro serialization with null fields."""
    task = Task(
        id=uuid.uuid4(),
        task_kind="test_task",
        worker_kind="test_worker",
        created_at=datetime.now(timezone.utc),
        # Explicitly set nullable fields to None
        started_at=None,
        completed_at=None,
        executed_by=None,
        input_data=None,
        output_data=None,
        is_error=None,
        priority=None,
        ttl_duration=None,
        otel_ctx_carrier=None,
    )

    # Convert to Avro bytes
    avro_bytes = task.avro_bytes

    # Convert back from Avro bytes
    deserialized = Task.from_avro_bytes(avro_bytes)

    # Check all fields match
    assert task.id == deserialized.id
    assert task.task_kind == deserialized.task_kind
    assert task.worker_kind == deserialized.worker_kind
    assert task.created_at.timestamp() == deserialized.created_at.timestamp()
    assert task.started_at == deserialized.started_at
    assert task.completed_at == deserialized.completed_at
    assert task.executed_by == deserialized.executed_by
    assert task.input_data == deserialized.input_data
    assert task.output_data == deserialized.output_data
    assert task.is_error == deserialized.is_error
    assert task.priority == deserialized.priority
    assert task.ttl_duration == deserialized.ttl_duration
    assert task.otel_ctx_carrier == deserialized.otel_ctx_carrier
