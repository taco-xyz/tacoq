"""Tests for the health check functionality of the ManagerClient.

These tests verify that the health check endpoint correctly reports
different states based on the server's response.
"""

import pytest
from aiohttp import ClientConnectorError
from aiohttp.client_reqrep import ConnectionKey
from aioresponses import aioresponses
from tacoq.relay import RelayClient, RelayStates

# =========================================
# Health Check Tests
# =========================================


@pytest.mark.asyncio
async def test_health_check_healthy(mock_relay_client: RelayClient):
    """Test that a 200 response from the health endpoint returns HEALTHY state."""
    with aioresponses() as m:
        m.get("http://test/health", status=200)  # type: ignore
        state = await mock_relay_client.check_health()
        assert state == RelayStates.HEALTHY


@pytest.mark.asyncio
async def test_health_check_unknown(mock_relay_client: RelayClient):
    """Test that a 500 response from the health endpoint returns UNKNOWN state."""
    with aioresponses() as m:
        # Mock multiple attempts since RetryClient is used for 500 errors
        m.get("http://test/health", status=500, body=b"{}", repeat=True)  # type: ignore
        state = await mock_relay_client.check_health()
        assert state == RelayStates.UNKNOWN


@pytest.mark.asyncio
async def test_health_check_not_reachable(mock_relay_client: RelayClient):
    """Test that a connection error returns NOT_REACHABLE state."""
    with aioresponses() as m:
        m.get(  # type: ignore
            "http://test/health",
            exception=ClientConnectorError(
                ConnectionKey("test", 80, False, None, None, None, None), OSError()
            ),
        )
        state = await mock_relay_client.check_health()
        assert state == RelayStates.NOT_REACHABLE
