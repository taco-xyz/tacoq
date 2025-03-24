import uuid
from datetime import datetime, timezone

import pytest
from tacoq.core.models.task_completed_update import TaskCompletedUpdate


@pytest.mark.unit
def test_task_completed_update_avro_serde():
    update = TaskCompletedUpdate(
        id=uuid.uuid4(),
        completed_at=datetime.now(timezone.utc),
        output_data=b"test output",
        is_error=0,
    )

    # Convert to Avro bytes
    avro_bytes = update.avro_bytes

    # Convert back from Avro bytes
    deserialized = TaskCompletedUpdate.from_avro_bytes(avro_bytes)

    # Check all fields match
    assert update.id == deserialized.id
    assert update.completed_at.timestamp() == deserialized.completed_at.timestamp()
    assert update.output_data == deserialized.output_data
    assert update.is_error == deserialized.is_error
