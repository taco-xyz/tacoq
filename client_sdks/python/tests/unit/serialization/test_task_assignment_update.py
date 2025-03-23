import uuid
from datetime import datetime, timezone

import pytest
from tacoq.core.models.task_assignment_update import TaskAssignmentUpdate


@pytest.mark.unit
def test_task_assignment_update_avro_serde():
    update = TaskAssignmentUpdate(
        id=uuid.uuid4(),
        task_kind="test_task",
        worker_kind="test_worker",
        created_at=datetime.now(timezone.utc),
        input_data=b"test input",
        priority=128,
        ttl_duration=3600,
        otel_ctx_carrier={"trace_id": "123"},
    )

    # Convert to Avro bytes
    avro_bytes = update.avro_bytes

    # Convert back from Avro bytes
    deserialized = TaskAssignmentUpdate.from_avro_bytes(avro_bytes)

    # Check all fields match
    assert update.id == deserialized.id
    assert update.task_kind == deserialized.task_kind
    assert update.worker_kind == deserialized.worker_kind
    assert update.created_at.timestamp() == deserialized.created_at.timestamp()
    assert update.input_data == deserialized.input_data
    assert update.priority == deserialized.priority
    assert update.ttl_duration == deserialized.ttl_duration
    assert update.otel_ctx_carrier == deserialized.otel_ctx_carrier
