from src.manager import ManagerClient, ManagerStates
import pytest


@pytest.mark.asyncio
async def test_health_check_client(manager_client: ManagerClient):
    """Tests whether the manager client can check the health of the manager at all."""

    health_state = await manager_client.check_health()

    assert (
        health_state == ManagerStates.HEALTHY
    ), f"Manager is not healthy. Current state: {health_state}"
