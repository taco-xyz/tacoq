from datetime import datetime
from typing import Callable, Awaitable, Optional, Dict

import asyncio
from broker import WorkerBrokerClient
from manager import ManagerClient
from models.task import Task, TaskInput, TaskOutput, TaskStatus

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

    _shutdown_event: asyncio.Event = asyncio.Event()
    """ Event that is set when the worker application is shutting down. """

    _shutdown_complete_event: asyncio.Event = asyncio.Event()
    """ Event that is set when the worker application has completed shutting down. """

    _active_tasks: set[asyncio.Task[None]] = set()
    """ The set of active tasks that this worker application is processing. """

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
            result = e.__str__()
            is_error = True

        # Stop timer
        completed_at = datetime.now()

        # Update task
        task.output_data = result
        task.is_error = is_error
        task.started_at = started_at
        task.completed_at = completed_at
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
        """Start the worker application."""
        return await self._entrypoint()

    async def _entrypoint(self):
        """Initialize and start listening for tasks."""
        await self._init_broker_client()
        print("Worker application initialized!")
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
            start = datetime.now()
            while not self._shutdown_event.is_set():
                try:
                    task, message = await asyncio.wait_for(
                        self._broker_client.listen().__anext__(),
                        timeout=1.0,  # Check shutdown signal every second
                    )
                    print("Received task", task.id, "at", datetime.now() - start)
                    # Task is created and added to the tracker pool
                    async_task = asyncio.create_task(self._execute_task(task, message))
                    self._active_tasks.add(async_task)
                    async_task.add_done_callback(self._active_tasks.discard)
                except asyncio.TimeoutError:
                    continue
            print(f"Waiting for {len(self._active_tasks)} active tasks to complete...")
            await asyncio.gather(
                *self._active_tasks
            )  # Wait for all active tasks to complete
        except asyncio.CancelledError:
            pass

    def issue_shutdown(self):
        """Shutdown the worker application."""
        print("Shutdown signal received!")
        self._shutdown_event.set()

    async def wait_for_shutdown(self):
        """Wait for the worker application to shut down."""
        print("Waiting for shutdown to complete...")
        await self._shutdown_complete_event.wait()

    async def _cleanup(self):
        """Cleanup the worker application.

        This method is called when the worker is shutting down. Used for cleaning internal state.
        """
        print("CLEANING UP...")
        if self._broker_client is not None:
            await self._broker_client.disconnect()
        print("CLEANUP COMPLETE")
        self._shutdown_complete_event.set()
