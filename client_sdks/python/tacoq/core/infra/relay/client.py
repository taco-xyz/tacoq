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

from pydantic import BaseModel
from aiohttp import ClientSession, ClientConnectorError
from aiohttp_retry import RetryClient, RetryOptionsBase
from opentelemetry.propagate import inject

from tacoq.core.infra.relay.config import RelayConfig
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
    """Abstracts the relay API.

    ### Attributes
    - config: The configuration for the relay client.

    ### Usage
    ```python
    # Initialize and use the client with async context relay
    relay = RelayClient(config=RelayConfig(url="http://localhost:8080"))
    await relay.connect()

    # Check the health of the relay
    health = await relay.check_health()

    # Get a task by its ID
    task = await relay.get_task(task.id)

    # Disconnect from the relay
    await relay.disconnect()
    ```
    """

    config: RelayConfig
    """Configuration for the relay client."""

    _session: Optional[ClientSession] = None
    """Internal aiohttp session."""

    @property
    async def session(self) -> ClientSession:
        """Get or create the client session."""
        if not self._session:
            await self.connect()
        return self._session  # type: ignore

    async def connect(self) -> None:
        """Connect to the relay."""
        if not self._session:
            self._session = ClientSession()

    async def disconnect(self) -> None:
        """Disconnect from the relay."""
        if self._session:
            await self._session.close()
            self._session = None

    # ================================
    # Health Checking
    # ================================

    async def check_health(
        self: Self, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> RelayStates:
        """Check whether the relay is healthy. This is currently used before
        tests are run to notify the user if the relay is not healthy or even
        running at all.

        ### Arguments:
        - override_retry_options: Retry options to override the default ones

        ### Returns:
        RelayStates: Whether the relay is healthy.
        """
        try:
            session = await self.session
            retry_client = RetryClient(
                session,
                retry_options=override_retry_options
                or self.config.default_retry_options,
            )
            async with retry_client.get(f"{self.config.url}{HEALTH_PATH}") as resp:
                match resp.status:
                    case 200:
                        return RelayStates.HEALTHY
                    case _:
                        return RelayStates.UNKNOWN
        except ClientConnectorError:
            return RelayStates.NOT_REACHABLE

    # ================================
    # Task Get/Set Operations
    # ================================

    async def get_task(
        self: Self,
        task_id: UUID,
        override_retry_options: Optional[RetryOptionsBase] = None,
    ) -> Optional[Task]:
        """Get a task by its UUID.

        ### Arguments:
        - task_id: UUID of the task to retrieve
        - override_retry_options: Retry options to override the default ones

        ### Returns:
        Task: The task details
        """
        tracer = TracerManager.get_tracer()
        with tracer.start_as_current_span("get_task") as span:
            span.set_attributes({"task.id": str(task_id)})

            # Inject context into headers so we can trace the request back to the relay
            headers: dict[str, str] = {}
            inject(headers)

            session = await self.session
            retry_client = RetryClient(
                session,
                retry_options=override_retry_options
                or self.config.default_retry_options,
            )

            async with retry_client.get(
                f"{self.config.url}{TASK_PATH}/{task_id}", headers=headers
            ) as resp:
                if resp.status == 404:
                    return None
                resp.raise_for_status()
                data = await resp.json()

                return Task(**data)
