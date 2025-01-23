from dataclasses import dataclass
from typing import Callable, Awaitable, Optional, Dict

import asyncio
from broker import WorkerBrokerClient
from manager import ManagerClient
from models.task import Task, TaskInput, TaskOutput

from worker.config import WorkerApplicationConfig


# =========================================
# Errors
# =========================================


class TaskNotRegistered(Exception):
    """Exception raised when a task is not registered."""

    def __init__(
        self,
        task_kind: str,
        registered_tasks: Dict[str, Callable[[TaskInput], Awaitable[TaskOutput]]],
    ):
        self.message = f"Task {task_kind} not registered for this worker. Available tasks: {registered_tasks.keys()}"
        super().__init__(self.message)


# =========================================
# Worker Application
# =========================================


@dataclass
class WorkerApplication:
    """A worker application that processes tasks from a task queue."""

    _config: WorkerApplicationConfig
    """ The configuration for this worker application. """

    _registered_tasks: Dict[str, Callable[[TaskInput], Awaitable[TaskOutput]]]
    """ All the tasks that this worker application can handle. """

    _broker_client: Optional[WorkerBrokerClient]
    """ The broker client that this worker application uses. """

    _queue_name: Optional[str]
    """ The queue name that this worker application listens to. """

    _manager_client: ManagerClient
    """ The manager client that this worker application uses to interface with the manager service. """

    def __init__(self, config: WorkerApplicationConfig):
        # Init all the clients
        self._config = config
        self._id = None
        self._manager_client = ManagerClient(config.manager_config)
        self._registered_tasks = {}

    # ================================
    # Task Registration & Execution
    # ================================

    def register_task(
        self, kind: str, task: Callable[[TaskInput], Awaitable[TaskOutput]]
    ):
        """Register a task handler function for a specific task kind.

        ### Parameters
        - `kind`: Unique identifier for the task type
        - `task`: Async function that processes tasks of this kind
        """
        self._registered_tasks[kind] = task

    def task(
        self, kind: str
    ) -> Callable[
        [Callable[[TaskInput], Awaitable[TaskOutput]]],
        Callable[[TaskInput], Awaitable[TaskOutput]],
    ]:
        """Decorator for registering task handler functions.

        ### Parameters
        - `kind`: Unique identifier for the task type

        ### Returns
        - `Callable`: Decorator function that registers the task handler
        """

        def decorator(
            task: Callable[[TaskInput], Awaitable[TaskOutput]],
        ) -> Callable[[TaskInput], Awaitable[TaskOutput]]:
            self.register_task(kind, task)
            return task

        return decorator

    async def _execute_task(self, task: Task):
        """Execute a task and update its status in the manager.

        ### Parameters
        - `kind`: Type of task to execute
        - `input_data`: Input data for the task
        - `task_id`: Unique identifier for the task

        ### Raises
        - `ValueError`: If task kind is not registered
        """

        # Find task handler
        task_func = self._registered_tasks.get(task.task_kind.name)
        if task_func is None:
            raise TaskNotRegistered(task.task_kind.name, self._registered_tasks)

        # Compute task result
        result: Optional[TaskOutput | Exception] = None
        is_error: bool = False

        try:
            result = await task_func(task.input_data)
        except Exception as e:
            result = Exception(e)
            is_error = True

        # TODO Submit task result via broker
        print(f"Task {task.task_kind.name} {is_error}: {result}")
        ...

    # ================================
    # Worker Lifecycle
    # ================================

    async def entrypoint(self):
        """Start the worker application.

        This method registers the worker kind with the manager,
        starts listening for tasks, and handles graceful shutdown.
        """

        # Register the worker kind with the manager
        worker_kind_info = await self._manager_client.get_worker_kind_broker_info(
            self._config.kind
        )
        self._queue_name = worker_kind_info.queue_name

        # Init the broker client using the queue name of the worker kind
        self._broker_client = WorkerBrokerClient(
            self._config.broker_config, self._queue_name
        )

        # Begin loop
        try:
            await self._listen()
        except asyncio.CancelledError:
            pass
        finally:
            await self._cleanup()

    async def _listen(self):
        """Listen for tasks of a specific kind from the broker.

        ### Raises
        - `RuntimeError`: If broker client is not initialized
        """

        if self._broker_client is None:
            raise RuntimeError("Broker client not initialized")

        try:
            async for task in self._broker_client.listen():
                await self._execute_task(task)
        except asyncio.CancelledError:
            await self._broker_client.disconnect()

    async def _cleanup(self):
        """Cleanup the worker application.

        This method is called when the worker is shutting down. Used for cleaning internal state.
        """
        # TODO - Add tracing and potential cleanup logic if needed
        ...
