import pytest
from aiohttp import ClientConnectorError
from aiohttp.client_reqrep import ConnectionKey
from aioresponses import aioresponses

from manager.client import ManagerClient, ManagerStates


@pytest.mark.asyncio
async def test_health_check_healthy(mock_manager_client: ManagerClient):
    with aioresponses() as m:
        m.get("http://test/health", status=200)  # type: ignore
        state = await mock_manager_client.check_health()
        assert state == ManagerStates.HEALTHY


@pytest.mark.asyncio
async def test_health_check_unknown(mock_manager_client: ManagerClient):
    with aioresponses() as m:
        # Mock multiple attempts since RetryClient is used for 500 errors
        m.get("http://test/health", status=500, body=b"{}", repeat=True)  # type: ignore
        state = await mock_manager_client.check_health()
        assert state == ManagerStates.UNKNOWN


@pytest.mark.asyncio
async def test_health_check_not_reachable(mock_manager_client: ManagerClient):
    with aioresponses() as m:
        m.get(  # type: ignore
            "http://test/health",
            exception=ClientConnectorError(
                ConnectionKey("test", 80, False, None, None, None, None), OSError()
            ),
        )
        state = await mock_manager_client.check_health()
        assert state == ManagerStates.NOT_REACHABLE
