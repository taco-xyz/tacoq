from enum import Enum
from typing import Optional
from dataclasses import dataclass

from uuid import UUID
import aiohttp as aio

from manager.config import ManagerConfig
from models.task import (
    TaskStatus,
    TaskInput,
    TaskOutput,
    TaskInstance,
)

WORKER_PATH = "/workers"
""" Base path for worker registration and unregistration endpoints."""

TASK_PATH = "/tasks"
""" Base path for task CRUD operations."""


class ManagerStates(str, Enum):
    """Possible states of the manager.

    ### States
    - `HEALTHY`: The manager is healthy.
    - `UNHEALTHY`: The manager is unhealthy.
    - `NOT_REACHABLE`: The manager is not reachable.
    - `UNKNOWN`: The manager is in an unknown state.
    """

    HEALTHY = "healthy"
    UNHEALTHY = "unhealthy"
    NOT_REACHABLE = "not_reachable"
    UNKNOWN = "unknown"


@dataclass
class ManagerClient:
    """Abstracts the manager API for worker registration and unregistration."""

    config: ManagerConfig

    # Check whether the manager is healthy

    async def check_health(self) -> ManagerStates:
        """Check whether the manager is healthy. This is currently used before
        tests are run to notify the user if the manager is not healthy or even
        running at all.

        ### Returns
        - `ManagerStates`: Whether the manager is healthy.
        """
        try:
            async with aio.ClientSession(timeout=self.config.timeout) as session:
                async with session.get(f"{self.config.url}/health") as resp:
                    match resp.status:
                        case 200:
                            return ManagerStates.HEALTHY
                        case 503:
                            return ManagerStates.UNHEALTHY
                        case _:
                            return ManagerStates.UNHEALTHY
        except aio.ClientConnectorError:
            return ManagerStates.NOT_REACHABLE

    # Task Get/Set Operations

    async def get_task(self, task_id: UUID) -> TaskInstance:
        """Get a task by its UUID.

        ### Parameters
        - `task_id`: UUID of the task to retrieve

        ### Returns
        - `TaskInstance`: The task details
        """
        async with aio.ClientSession(timeout=self.config.timeout) as session:
            async with session.get(f"{self.config.url}{TASK_PATH}/{task_id}") as resp:
                resp.raise_for_status()
                data = await resp.json()
                return TaskInstance.from_dict(data)

    async def publish_task(
        self, task_kind_name: str, input_data: Optional[TaskInput] = None
    ) -> TaskInstance:
        """Create a new task.

        ### Parameters
        - `task_kind_name`: Name of the task kind
        - `input_data`: Optional input data for the task

        ### Returns
        - `TaskInstance`: The created task details
        """
        async with aio.ClientSession(timeout=self.config.timeout) as session:
            async with session.post(
                f"{self.config.url}{TASK_PATH}",
                json={"task_kind_name": task_kind_name, "input_data": input_data},
            ) as resp:
                resp.raise_for_status()
                data = await resp.json()
                return TaskInstance.from_dict(data)

    async def update_task_status(self, task_id: UUID, status: TaskStatus) -> None:
        """Update the status of a task.

        ### Parameters
        - `task_id`: UUID of the task to update
        - `status`: New status for the task
        """
        async with aio.ClientSession(timeout=self.config.timeout) as session:
            async with session.put(
                f"{self.config.url}{TASK_PATH}/{task_id}/status", json=status.value
            ) as resp:
                resp.raise_for_status()

    async def update_task_result(
        self, task_id: UUID, data: TaskOutput, is_error: bool = False
    ) -> None:
        """Submit results or error for a task.

        ### Parameters
        - `task_id`: UUID of the task to update
        - `data`: Result data or error message
        - `is_error`: Whether this is an error result
        """
        async with aio.ClientSession(timeout=self.config.timeout) as session:
            async with session.put(
                f"{self.config.url}{TASK_PATH}/{task_id}/result",
                json={"data": data, "is_error": is_error},
            ) as resp:
                resp.raise_for_status()

    # Worker registration and unregistration

    async def register_worker(self, name: str, task_kinds: list[str]) -> UUID:
        """Register a new worker with the manager service. Called on worker startup.

        ### Parameters
        - `name`: The name of the worker.
        - `task_kinds`: The task kinds that the worker can handle.

        ### Returns
        - `UUID`: The ID of the registered worker.
        """

        async with aio.ClientSession(timeout=self.config.timeout) as session:
            async with session.post(
                f"{self.config.url}{WORKER_PATH}",
                json={"name": name, "task_kinds": task_kinds},
            ) as resp:
                resp.raise_for_status()
                data = await resp.json()
                return UUID(data["id"])

    async def unregister_worker(self, worker_id: UUID) -> None:
        """Unregister an existing worker. Called on graceful worker shutdown.

        ### Parameters
        - `worker_id`: The ID of the worker to unregister.
        """
        async with aio.ClientSession(timeout=self.config.timeout) as session:
            async with session.delete(
                f"{self.config.url}{WORKER_PATH}/{worker_id}"
            ) as resp:
                resp.raise_for_status()
