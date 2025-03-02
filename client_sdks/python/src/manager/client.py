from enum import Enum
from typing import Optional
from uuid import UUID

from pydantic import BaseModel
from aiohttp import ClientSession, ClientConnectorError
from aiohttp_retry import RetryClient, RetryOptionsBase
from opentelemetry.propagate import inject

from manager.config import ManagerConfig
from models.task import Task
from tracer_manager import TracerManager

# =========================================
# Constants
# =========================================

TASK_PATH = "/tasks"
""" Base path for task CRUD operations."""

HEALTH_PATH = "/health"
""" Base path for health checking."""

# =========================================
# Manager States
# =========================================


class ManagerStates(str, Enum):
    """Possible states of the manager. Used for health checking during tests."""

    HEALTHY = "healthy"
    """ The manager is healthy. """

    UNHEALTHY = "unhealthy"
    """ The manager is unhealthy. """

    NOT_REACHABLE = "not_reachable"
    """ The manager is not reachable. """

    UNKNOWN = "unknown"
    """ The manager is in an unknown state. Schrödinger's Manager?"""


class ManagerClient(BaseModel):
    """Abstracts the manager API."""

    config: ManagerConfig
    """Configuration for the manager client."""

    # ================================
    # Health Checking
    # ================================

    async def check_health(
        self, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> ManagerStates:
        """Check whether the manager is healthy. This is currently used before
        tests are run to notify the user if the manager is not healthy or even
        running at all.

        ### Args
        - `override_retry_options`: Retry options to override the default ones

        ### Returns
        - `ManagerStates`: Whether the manager is healthy.
        """

        try:
            async with ClientSession() as session:
                retry_client = RetryClient(
                    session,
                    retry_options=override_retry_options or self.config.retry_options,
                )
                async with retry_client.get(f"{self.config.url}{HEALTH_PATH}") as resp:
                    match resp.status:
                        case 200:
                            return ManagerStates.HEALTHY
                        case _:
                            return ManagerStates.UNKNOWN
        except ClientConnectorError:
            return ManagerStates.NOT_REACHABLE

    # ================================
    # Task Get/Set Operations
    # ================================

    async def get_task(
        self, task_id: UUID, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> Optional[Task]:
        """Get a task by its UUID.

        ### Args:
        - task_id: UUID of the task to retrieve
        - override_retry_options: Retry options to override the default ones

        ### Returns
        - Task: The task details
        """
        tracer = TracerManager.get_tracer()
        with tracer.start_as_current_span("get_task") as span:
            span.set_attributes({"task.id": str(task_id)})

            # Inject context into headers so we can trace the request back to the manager
            headers: dict[str, str] = {}
            inject(headers)

            async with ClientSession() as session:
                retry_client = RetryClient(
                    session,
                    retry_options=override_retry_options or self.config.retry_options,
                )

                async with retry_client.get(
                    f"{self.config.url}{TASK_PATH}/{task_id}", headers=headers
                ) as resp:
                    if resp.status == 404:
                        return None
                    resp.raise_for_status()
                    data = await resp.json()

                    return Task(**data)
