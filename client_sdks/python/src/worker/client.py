from datetime import datetime
from typing import Callable, Awaitable, Optional, Dict

import asyncio
from broker import WorkerBrokerClient
from manager import ManagerClient
from models.task import Task, TaskInput, TaskOutput, TaskResult, TaskStatus

from pydantic import BaseModel
from worker.config import WorkerApplicationConfig

from aio_pika.abc import (
    AbstractIncomingMessage,
)

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


class SerializableException(BaseModel):
    """A serializable exception."""

    type: str
    """ The type of the exception. `RuntimeError` evaluates to `"RuntimeError"`."""

    message: str
    """ The message of the exception. """


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

    _shutdown: bool = False
    """ Whether the worker application is shutting down. """

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

    async def _execute_task(self, task: Task, message: AbstractIncomingMessage):
        """Execute a task and update its status in the manager.

        ### Parameters
        - `task`: Task to execute

        ### Raises
        - `ValueError`: If task kind is not registered
        """

        # Check if broker client is initialized
        if self._broker_client is None:
            raise RuntimeError("Broker client not initialized")

        # Find task handler
        task_func = self._registered_tasks.get(task.task_kind)
        if task_func is None:
            raise TaskNotRegisteredError(task.task_kind, self._registered_tasks)

        # Compute task result
        result: Optional[TaskOutput | Exception] = None
        is_error: bool = False

        # Start timer
        started_at = datetime.now()

        # Execute task
        try:
            result = await task_func(task.input_data)
        except Exception as e:
            result = SerializableException(
                type=e.__class__.__name__,
                message=e.__str__(),
            )
            is_error = True

        # Stop timer
        completed_at = datetime.now()

        # Update task
        task.result = TaskResult(
            data=result,
            is_error=is_error,
            started_at=started_at,
            completed_at=completed_at,
        )
        task.status = TaskStatus.COMPLETED

        # Submit task result via broker
        await self._broker_client.publish_task_result(
            task=task,
        )

        # Acknowledge the message
        await message.ack()

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

        # Init the broker client using the queue name of the worker kind
        self._broker_client = WorkerBrokerClient(
            config=self.config.broker_config,
            worker_kind=self.config.kind,
        )
        await self._broker_client.connect()

    async def entrypoint(self):
        if self.config.workers == 1:
            return await self._entrypoint()

        import multiprocessing
        import sys

        # Create worker processes
        processes: list[multiprocessing.Process] = []
        for _ in range(self.config.workers):
            process = multiprocessing.Process(
                target=asyncio.run, args=(self._entrypoint(),)
            )
            process.start()
            processes.append(process)

        # Wait for all processes to complete
        for process in processes:
            process.join()

        # Exit with error if any process failed
        for process in processes:
            if process.exitcode != 0:
                sys.exit(1)
        return

    async def _entrypoint(self):
        """Start the worker application.

        This method registers the worker kind with the manager,
        starts listening for tasks, and handles graceful shutdown.
        """
        await self._init_broker_client()

        print("Worker application initialized!")

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
            async for task, message in self._broker_client.listen():
                if self._shutdown:
                    break
                asyncio.create_task(self._execute_task(task, message))
        except asyncio.CancelledError:
            pass
        finally:
            await self._cleanup()

    def shutdown(self):
        """Shutdown the worker application."""
        self._shutdown = True

    async def _cleanup(self):
        """Cleanup the worker application.

        This method is called when the worker is shutting down. Used for cleaning internal state.
        """
        if self._broker_client is not None:
            await self._broker_client.disconnect()
