import uuid
from datetime import datetime, timezone

import pytest
from tacoq.core.models.task_running_update import TaskRunningUpdate


@pytest.mark.unit
def test_task_running_update_avro_serde():
    update = TaskRunningUpdate(
        id=uuid.uuid4(),
        started_at=datetime.now(timezone.utc),
        executed_by="test_worker",
    )

    # Convert to Avro bytes
    avro_bytes = update.avro_bytes

    # Convert back from Avro bytes
    deserialized = TaskRunningUpdate.from_avro_bytes(avro_bytes)

    # Check all fields match
    assert update.id == deserialized.id
    assert update.started_at.timestamp() == deserialized.started_at.timestamp()
    assert update.executed_by == deserialized.executed_by
