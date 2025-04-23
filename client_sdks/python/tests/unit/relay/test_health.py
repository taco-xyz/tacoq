"""Tests for the health check functionality of the ManagerClient.

These tests verify that the health check endpoint correctly reports
different states based on the server's response.
"""

import pytest
from unittest.mock import AsyncMock
from tacoq.relay import RelayStates

# =========================================
# Health Check Tests
# =========================================


@pytest.mark.asyncio
async def test_health_check_healthy(
    mock_relay_client: AsyncMock,
):  # Use AsyncMock type hint
    """Test that the client correctly interprets a healthy state."""
    # Configure mock return value
    mock_relay_client.check_health.return_value = RelayStates.HEALTHY

    # Call the method on the mock
    state = await mock_relay_client.check_health()

    # Assert the result
    assert state == RelayStates.HEALTHY
    # Assert the mock was called
    mock_relay_client.check_health.assert_awaited_once()


@pytest.mark.asyncio
async def test_health_check_unknown(
    mock_relay_client: AsyncMock,
):  # Use AsyncMock type hint
    """Test that the client correctly interprets an unknown state."""
    # Configure mock return value
    mock_relay_client.check_health.return_value = RelayStates.UNKNOWN

    # Call the method on the mock
    state = await mock_relay_client.check_health()

    # Assert the result
    assert state == RelayStates.UNKNOWN
    # Assert the mock was called
    mock_relay_client.check_health.assert_awaited_once()


@pytest.mark.asyncio
async def test_health_check_not_reachable(
    mock_relay_client: AsyncMock,
):  # Use AsyncMock type hint
    """Test that the client correctly interprets a not reachable state."""
    # Configure mock return value
    mock_relay_client.check_health.return_value = RelayStates.NOT_REACHABLE

    # Call the method on the mock
    state = await mock_relay_client.check_health()

    # Assert the result
    assert state == RelayStates.NOT_REACHABLE
    # Assert the mock was called
    mock_relay_client.check_health.assert_awaited_once()
