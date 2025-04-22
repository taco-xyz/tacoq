"""Relay client for the TacoQ client SDK.

The relay client abstracts the details of communicating with
the relay service. This class is not meant to be used directly
by the user. Instead, they should refer to the `PublisherClient`
to fetch existing tasks or publish new ones.
"""

from enum import Enum
from typing import Optional
from uuid import UUID
from typing_extensions import Self
from typing import Any
from pydantic import BaseModel
from opentelemetry.propagate import inject

import httpx
from httpx import AsyncHTTPTransport, AsyncClient

from tacoq.relay.config import RelayConfig
from tacoq.core.models.task import Task
from tacoq.core.telemetry import TracerManager

# =========================================
# Constants
# =========================================

TASK_PATH = "/tasks"
""" Base path for task CRUD operations."""

HEALTH_PATH = "/health"
""" Base path for health checking."""

# =========================================
# Relay States
# =========================================


class RelayStates(str, Enum):
    """Possible states of the relay. Used primarly for health checking during
    tests."""

    HEALTHY = "healthy"
    """ The relay is healthy. """

    UNHEALTHY = "unhealthy"
    """ The relay is unhealthy. """

    NOT_REACHABLE = "not_reachable"
    """ The relay is not reachable. """

    UNKNOWN = "unknown"
    """ The relay is in an unknown state. Schrödinger's Relay?"""


class RelayClient(BaseModel):
    """Abstracts the relay API. Allows for getting task results from the relay
    and performing health checks.

    ### Attributes
    - config: The configuration for the relay client.

    ### Usage
    ```python
    # Initialize and use the client with async context relay
    relay = RelayClient(config=RelayConfig(url="http://localhost:8080"))

    # Check the health of the relay
    health = await relay.check_health()

    # Get a task by its ID
    task = await relay.get_task(task.id)

    # Disconnect from the relay
    await relay.disconnect()
    ```
    It can also be used as a context manager:
    ```python
    async with RelayClient(config=RelayConfig(url="http://localhost:8080")) as relay:
        # Check the health of the relay
        health = await relay.check_health()
    ```
    """

    config: RelayConfig
    """Configuration for the relay client."""

    _client: Optional[AsyncClient] = None
    """Internal aiohttp session."""

    @property
    async def client(self) -> AsyncClient:
        """Get or create the client session."""
        if not self._client:
            await self.connect()
        return self._client  # type: ignore

    async def connect(self) -> None:
        """Connect to the relay."""
        if not self._client:
            transport = AsyncHTTPTransport(
                verify=self.config.ssl_verify, http2=True, retries=self.config.retries
            )
            self._client = AsyncClient(http2=True, transport=transport)

    async def disconnect(self) -> None:
        """Disconnect from the relay."""
        if self._client:
            await self._client.aclose()
            self._client = None

    # ================================
    # Context Management
    # ================================

    async def __aenter__(self: Self) -> Self:
        await self.connect()
        return self

    async def __aexit__(
        self: Self, exc_type: Any, exc_value: Any, traceback: Any
    ) -> None:
        await self.disconnect()

    # ================================
    # Health Checking
    # ================================

    async def check_health(self: Self) -> RelayStates:
        """Check whether the relay is healthy. This is currently used before
        tests are run to notify the user if the relay is not healthy or even
        running at all.

        ### Arguments:
        - override_retry_options: Retry options to override the default ones

        ### Returns:
        RelayStates: Whether the relay is healthy.
        """
        try:
            client = await self.client
            resp = await client.get(
                f"{self.config.url}{HEALTH_PATH}", timeout=self.config.timeout
            )

            match resp.status_code:
                case 200:
                    return RelayStates.HEALTHY
                case _:
                    return RelayStates.UNKNOWN
        except httpx.ConnectError:
            return RelayStates.NOT_REACHABLE
        except (httpx.RequestError, Exception):
            return RelayStates.UNKNOWN

    # ================================
    # Task Get/Set Operations
    # ================================

    async def get_task(
        self: Self,
        task_id: UUID,
    ) -> Optional[Task]:
        """Get a task by its UUID.

        ### Arguments:
        - task_id: UUID of the task to retrieve
        - override_retry_options: Retry options to override the default ones

        ### Returns:
        Task: The task details

        ### Example:
        ```python
        task = await relay.get_task(task_id)
        ```
        """

        tracer = TracerManager.get_tracer()
        with tracer.start_as_current_span("get_task") as span:
            span.set_attributes({"task.id": str(task_id)})

            # Inject context into headers so we can trace the request back to the relay
            headers: dict[str, str] = {}
            inject(headers)
            headers["Accept"] = "application/avro"

            client = await self.client
            resp = await client.get(
                f"{self.config.url}{TASK_PATH}/{task_id}",
                headers=headers,
                timeout=self.config.timeout,
            )

            if resp.status_code == 404:
                return None

            resp.raise_for_status()
            data = resp.content
            return Task.from_avro_bytes(data)
