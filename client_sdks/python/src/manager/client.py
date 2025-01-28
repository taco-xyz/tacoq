from enum import Enum
from typing import Optional
from uuid import UUID

from pydantic import BaseModel
from aiohttp import ClientSession, ClientConnectorError
from aiohttp_retry import RetryClient, RetryOptionsBase

from manager.config import ManagerConfig
from models.task import Task, WorkerKindBrokerInfo

# =========================================
# Constants
# =========================================

TASK_PATH = "/tasks"
""" Base path for task CRUD operations."""

WORKER_KIND_PATH = "/worker-kind"
""" Base path for worker kind registration."""

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

    worker_kind_broker_info: dict[str, WorkerKindBrokerInfo] = {}
    """Cache of worker kind broker information. Read 
    `get_worker_kind_broker_info()` for more details."""

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
    ) -> Task:
        """Get a task by its UUID.

        ### Args
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
                f"{self.config.url}{TASK_PATH}/{task_id}"
            ) as resp:
                resp.raise_for_status()
                data = await resp.json()
                return Task(**data)

    # ================================
    # Worker Information
    # ================================

    async def get_worker_kind_broker_info(
        self, worker_kind: str
    ) -> WorkerKindBrokerInfo:
        """Get the broker information for a worker kind. Checks locally before
        fetching from the manager.

        ### Usage
        The publisher and the worker use this information
        in different ways:
        - The publisher uses the routing key to correctly route the task to the
        appropriate queues. This information is fetched from the manager the first
        time a new worker kind is used in task submission.
        - The worker uses the queue name to consume the correct queue. This information
        is fetched from the manager on startup, and the worker will only connect to the
        queue once it gets a response from the manager.


        ### Constraints
        This information could be computed within the SDK, but for the sake of consistency
        and easier portability to each language, we let the manager handle this logic instead.
        The manager also handles queue and exchange creation, so the SDKs can be lightweight.

        This means the entire system hinges on the manager being required for any of the
        SDKs to start up, which is something we want to address in the future. One idea
        for addressing this is having a Rust-based library that each SDK can use to set
        up the necessary queues and exchanges, and the logic is shared across all languages.

        ### Args:
        - `worker_kind`: The worker kind to register

        ### Returns
        - `WorkerKindBrokerInfo`: Information about the worker kind.
        """

        if worker_kind not in self.worker_kind_broker_info:
            self.worker_kind_broker_info[
                worker_kind
            ] = await self._register_worker_kind(worker_kind)

        return self.worker_kind_broker_info[worker_kind]

    async def _register_worker_kind(self, worker_kind: str) -> WorkerKindBrokerInfo:
        """Registers a worker kind and fetches the information needed to
        interact with the broker.

        ### Args
        - `worker_kind`: The worker kind to register

        ### Returns
        - `WorkerKindBrokerInfo`: Information about the worker kind.
        """

        async with ClientSession() as session:
            retry_client = RetryClient(
                session,
                retry_options=self.config.retry_options,
            )

            async with retry_client.put(
                f"{self.config.url}{WORKER_KIND_PATH}/{worker_kind}"
            ) as resp:
                resp.raise_for_status()
                data = await resp.json()
                return WorkerKindBrokerInfo(**data)
