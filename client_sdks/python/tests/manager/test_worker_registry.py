from src.manager import ManagerClient
import pytest
from uuid import UUID


@pytest.mark.asyncio
async def test_worker_lifecycle(manager_client: ManagerClient):
    """Tests worker registration and unregistration."""

    # Register worker
    worker_name = "test_worker"
    task_kinds = ["test_task", "another_task"]
    worker_id = await manager_client.register_worker(worker_name, task_kinds)

    assert isinstance(worker_id, UUID)

    # Unregister worker
    await manager_client.unregister_worker(worker_id)


@pytest.mark.asyncio
async def test_unregister_nonexistent_worker(manager_client: ManagerClient):
    """Tests unregistering a worker that doesn't exist."""

    nonexistent_id = UUID("00000000-0000-0000-0000-000000000000")
    with pytest.raises(Exception):
        await manager_client.unregister_worker(nonexistent_id)
