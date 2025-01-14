from enum import Enum
from typing import Optional
from uuid import UUID

from pydantic import BaseModel
from aiohttp import ClientSession, ClientConnectorError
from aiohttp_retry import RetryClient, RetryOptionsBase

from manager.config import ManagerConfig
from models.task import Task

TASK_PATH = "/tasks"
""" Base path for task CRUD operations."""

HEALTH_PATH = "/health"
""" Base path for health checking."""


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
    """Abstracts the manager API task retrieval."""

    config: ManagerConfig

    # Check whether the manager is healthy

    async def check_health(
        self, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> ManagerStates:
        """Check whether the manager is healthy. This is currently used before
        tests are run to notify the user if the manager is not healthy or even
        running at all.

        ### Parameters
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
                async with retry_client.get(
                    f"{self.config.base_url}{HEALTH_PATH}"
                ) as resp:
                    match resp.status:
                        case 200:
                            return ManagerStates.HEALTHY
                        case _:
                            return ManagerStates.UNKNOWN
        except ClientConnectorError:
            return ManagerStates.NOT_REACHABLE

    # Task Get/Set Operations

    async def get_task(
        self, task_id: UUID, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> Task:
        """Get a task by its UUID.

        ### Parameters
        - `task_id`: UUID of the task to retrieve
        - `override_retry_options`: Retry options to override the default ones

        ### Returns
        - `Task`: The task details
        """

        async with ClientSession() as session:
            retry_client = RetryClient(
                session,
                retry_options=override_retry_options or self.config.retry_options,
            )

            async with retry_client.get(
                f"{self.config.base_url}{TASK_PATH}/{task_id}"
            ) as resp:
                resp.raise_for_status()
                data = await resp.json()
                return Task(**data)
