from dataclasses import dataclass
from typing import Callable, Awaitable, Optional, Dict

import asyncio
from uuid import UUID
from broker import create_broker_instance, BrokerClient
from manager import ManagerClient
from models.task import TaskInput, TaskOutput

from worker.config import WorkerApplicationConfig


@dataclass
class WorkerApplication:
    """A worker application that processes tasks from a task queue."""

    _config: WorkerApplicationConfig
    _registered_tasks: Dict[str, Callable[[TaskInput], Awaitable[TaskOutput]]]
    _broker_client: Optional[BrokerClient]
    _manager_client: ManagerClient
    _queue_name: str

    def __init__(self, config: WorkerApplicationConfig):
        # Init all the clients
        self._config = config
        self._id = None
        self._manager_client = ManagerClient(config.manager_config)
        self._registered_tasks = {}

        # Get the broker information for this worker kind
        worker_kind_info = asyncio.run(
            self._manager_client.get_worker_kind_broker_info(self._config.kind)
        )
        self._queue_name = worker_kind_info.queue_name

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

    async def _register_worker(self):
        """Register this worker with the manager and initialize broker connection.

        ### Raises
        - `ConnectionError`: If connection to manager or broker fails
        """
        worker = await self._manager_client.register_worker(
            self._config.name, list(self._registered_tasks.keys())
        )
        self._id = worker

        # For this ideally we would get the broker information from the manager
        self._broker_client = create_broker_instance(
            self._config.broker_config, self._config.name, str(self._id)
        )
        await self._broker_client.connect()

    async def _unregister_worker(self):
        """Unregister from the manager and clean up broker connection.

        ### Raises
        - `ValueError`: If worker is not registered
        """
        if self._id is None:
            raise ValueError("Worker is not registered.")

        try:
            if self._broker_client:
                await self._broker_client.disconnect()
        except Exception as e:
            raise ValueError(f"Error during broker disconnect: {e}")

        try:
            await self._manager_client.unregister_worker(self._id)
        except Exception as e:
            raise ValueError(f"Error during worker unregister: {e}")

        # Clear local variables state
        # Important for hot reloading code
        self.cleanup()

    async def _execute_task(self, kind: str, input_data: TaskInput, task_id: UUID):
        """Execute a task and update its status in the manager.

        ### Parameters
        - `kind`: Type of task to execute
        - `input_data`: Input data for the task
        - `task_id`: Unique identifier for the task

        ### Raises
        - `ValueError`: If task kind is not registered
        """
        task_func = self._registered_tasks.get(kind)
        if task_func is None:
            raise ValueError(f"Task {kind} not registered.")

        try:
            # Check what to do with the task result
            result = await task_func(input_data)
            await self._manager_client.update_task_result(
                task_id, result, is_error=False
            )
        except Exception as e:
            # Log the exception (could improve error handling)
            await self._manager_client.update_task_result(
                task_id, {"error": str(e)}, is_error=True
            )

    async def _listen(self):
        """Listen for tasks of a specific kind from the broker.

        ### Raises
        - `RuntimeError`: If broker client is not initialized
        """
        if not self._broker_client:
            raise RuntimeError("Broker client is not initialized.")

        input_data: TaskInput
        task_id: UUID
        task_kind: str

        try:
            async for (
                input_data,
                task_id,
                task_kind,
            ) in self._broker_client.listen():
                await self._execute_task(task_kind, input_data, task_id)
        except asyncio.CancelledError:
            await self._broker_client.disconnect()

    async def entrypoint(self):
        """Start the worker application.

        This method registers the worker, starts listening for tasks,
        and handles graceful shutdown.
        """
        await self._register_worker()

        try:
            await self._listen()
        except asyncio.CancelledError:
            pass
        finally:
            await self._unregister_worker()

    def cleanup(self):
        """Cleanup the worker application.

        This method is called when the worker is shutting down. Used for cleaning internal state.
        """
        self._broker_client = None
        self._registered_tasks = {}
        self._id = None
