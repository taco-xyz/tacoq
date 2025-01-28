from typing import Callable, Awaitable, Optional, Dict

import asyncio
from broker import WorkerBrokerClient
from manager import ManagerClient
from models.task import Task, TaskInput, TaskOutput

from pydantic import BaseModel
from worker.config import WorkerApplicationConfig


# =========================================
# Errors
# =========================================


class TaskNotRegisteredError(Exception):
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


class WorkerApplication(BaseModel):
    """A worker application that processes tasks from a task queue."""

    config: WorkerApplicationConfig
    """ The configuration for this worker application. """

    _manager_client: Optional[ManagerClient] = None
    """ The manager client that this worker application uses to interface with the manager service. """

    _registered_tasks: Dict[str, Callable[[TaskInput], Awaitable[TaskOutput]]] = {}
    """ All the tasks that this worker application can handle. """

    _broker_client: Optional[WorkerBrokerClient] = None
    """ The broker client that this worker application uses. """

    _queue_name: Optional[str] = None
    """ The queue name that this worker application listens to. """

    def model_post_init(self, _) -> None:
        self._registered_tasks = {}
        self._manager_client = ManagerClient(config=self.config.manager_config)

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
        task_func = self._registered_tasks.get(task.task_kind)
        if task_func is None:
            raise TaskNotRegisteredError(task.task_kind, self._registered_tasks)

        # Compute task result
        result: Optional[TaskOutput | Exception] = None
        is_error: bool = False

        try:
            result = await task_func(task.input_data)
        except Exception as e:
            result = Exception(e)
            is_error = True

        # TODO Submit task result via broker
        print(f"Task {task.task_kind} {is_error}: {result}")
        ...

    # ================================
    # Worker Lifecycle
    # ================================

    async def _init_broker_client(self):
        """Initialize the broker client for this worker.

        ### Raises
        - `RuntimeError`: If manager client is not initialized
        """
        if self._manager_client is None:
            raise RuntimeError("Manager client not initialized")

        # Register the worker kind with the manager
        worker_kind_info = await self._manager_client.get_worker_kind_broker_info(
            self.config.kind
        )
        self._queue_name = worker_kind_info.queue_name

        # Init the broker client using the queue name of the worker kind
        self._broker_client = WorkerBrokerClient(
            config=self.config.broker_config,
            task_assignment_queue_name=self._queue_name,
        )
        await self._broker_client.connect()

    async def entrypoint(self):
        """Start the worker application.

        This method registers the worker kind with the manager,
        starts listening for tasks, and handles graceful shutdown.
        """
        await self._init_broker_client()

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
