import asyncio
import json
from datetime import datetime
from typing import Awaitable, Callable, Dict, Optional

from aio_pika.abc import (
    AbstractIncomingMessage,
)
from broker import WorkerBrokerClient
from logger_manager import LoggerManager
from logger_manager import StructuredMessage as _
from manager import ManagerClient
from models.exception import SerializedException
from models.task import Task, TaskInput, TaskOutput, TaskStatus
from opentelemetry.propagate import extract
from opentelemetry.trace import Status, StatusCode
from pydantic import BaseModel
from tracer_manager import TracerManager
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

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message=f"Registering task of kind {kind} with handler {task.__name__}",
                attributes={
                    "kind": kind,
                    "handler": task.__name__,
                },
            )
        )

        self._registered_tasks[kind] = task

    def task(
        self, kind: str
    ) -> Callable[
        [Callable[[TaskInput], Awaitable[TaskOutput]]],
        Callable[[TaskInput], Awaitable[TaskOutput]],
    ]:
        """Decorator for registering task handler functions.

        ### Arguments
        - kind: Unique identifier for the task type

        ### Returns
        - Callable: Decorator function that registers the task handler
        """

        def decorator(
            task: Callable[[TaskInput], Awaitable[TaskOutput]],
        ) -> Callable[[TaskInput], Awaitable[TaskOutput]]:
            self.register_task(kind, task)
            return task

        return decorator

    async def _execute_task(self, task: Task, message: AbstractIncomingMessage):
        """Execute a task and update its status in the manager.

        ### Arguments
        - task: Task to execute
        - message: Message to acknowledge
        """

        # Check if broker client is initialized
        if self._broker_client is None:
            raise RuntimeError("Broker client not initialized")

        # Extract OTEL context from the task and init loggers and tracers

        if task.otel_ctx_carrier is not None:
            otel_ctx_carrier = extract(task.otel_ctx_carrier)
        else:
            otel_ctx_carrier = None

        tracer = TracerManager.get_tracer()
        logger = LoggerManager.get_logger()

        with tracer.start_as_current_span(
            "task_worker_lifecycle",
            context=otel_ctx_carrier,
            attributes={
                "task.id": str(task.id),
                "task.kind": task.task_kind,
                "worker.kind": self.config.kind,
            },
        ) as parent_span:
            # Find task handler. If it doesn't exist, we nack the message and return
            # If this task is not registered anywhere, it will loop infinitely through
            # the workers. That's the user's problem.

            task_func = self._registered_tasks.get(task.task_kind)
            if task_func is None:
                error = TaskNotRegisteredError(task.task_kind, self._registered_tasks)
                parent_span.set_status(Status(StatusCode.ERROR))
                parent_span.record_exception(error)
                logger.error(
                    _(
                        message=f"Task of kind {task.task_kind} not registered. Available tasks: {self._registered_tasks.keys()}",
                        attributes={
                            "task.kind": task.task_kind,
                            "available_tasks": list(self._registered_tasks.keys()),
                        },
                    )
                )
                await message.nack()
                return

            # Task Execution ================================
            # TODO - Improve exception serialization

            result: Optional[TaskOutput] = None
            is_error: bool = False

            # Start timer
            started_at = datetime.now()
            parent_span.set_attribute("task.started_at", started_at.isoformat())

            with tracer.start_as_current_span(
                "task_execution",
                attributes={
                    "task.handler": task_func.__name__,
                    "task.input_size": len(str(task.input_data)),
                },
            ) as execution_span:
                try:
                    result = await task_func(task.input_data)
                    execution_span.set_attribute("task.output_size", len(str(result)))
                except Exception as e:
                    result = json.dumps(
                        SerializedException.from_exception(e).model_dump()
                    )
                    is_error = True
                    logger.error(
                        _(
                            message=f"Error executing task of kind {task.task_kind} with ID {task.id}",
                            attributes={
                                "task.id": str(task.id),
                                "task.kind": task.task_kind,
                            },
                        )
                    )
                    execution_span.set_status(Status(StatusCode.ERROR))
                    execution_span.record_exception(e)

            # Stop timer
            completed_at = datetime.now()

            # Update task
            task.output_data = result
            task.is_error = is_error
            task.started_at = started_at
            task.completed_at = completed_at
            task.status = TaskStatus.COMPLETED

            # Submission =========================================

            # Submit task output via broker
            with tracer.start_as_current_span(
                "publish_task_result",
                attributes={
                    "task.result_size": len(str(result)),
                    "task": str(task.model_dump()),
                },
            ):
                await self._broker_client.publish_task_result(task=task)

            # Acknowledge the message
            with tracer.start_as_current_span("acknowledge_message"):
                await message.ack()

            # Set span status based on whether the task was successful or not
            status = Status(StatusCode.OK) if not is_error else Status(StatusCode.ERROR)
            parent_span.set_status(status)

    # ================================
    # Worker Lifecycle
    # ================================

    # Entrypoint

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
            prefetch_count=self.config.broker_prefetch_count,
        )
        await self._broker_client.connect()

    async def entrypoint(self):
        """Start the worker application."""
        return await self._entrypoint()

    async def _entrypoint(self):
        """Initialize and start listening for tasks."""
        await self._init_broker_client()

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Worker application initialized!",
                attributes={"worker.kind": self.config.kind},
            )
        )
        try:
            await self._listen()
        except asyncio.CancelledError:
            pass
        finally:
            await self._cleanup()

    # Loop

    async def _listen(self):
        """Listen for tasks of a specific kind from the broker.

        ### Raises
        - `RuntimeError`: If broker client is not initialized
        """

        if self._broker_client is None:
            raise RuntimeError("Broker client not initialized")

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Listening for tasks",
            )
        )
        try:
            while not self._shutdown_event.is_set():
                try:
                    task, message = await asyncio.wait_for(
                        self._broker_client.listen().__anext__(),
                        timeout=1.0,  # Check shutdown signal every second
                    )
                    # Task is created and added to the tracker pool
                    logger.info(
                        _(
                            message=f"Received task of kind {task.task_kind} with ID {task.id}",
                            attributes={
                                "task.id": str(task.id),
                                "task.kind": task.task_kind,
                            },
                        )
                    )
                    async_task = asyncio.create_task(self._execute_task(task, message))
                    self._active_tasks.add(async_task)
                    async_task.add_done_callback(self._active_tasks.discard)
                except asyncio.TimeoutError:
                    continue
            logger = LoggerManager.get_logger()
            logger.info(
                _(
                    message=f"Waiting for {len(self._active_tasks)} active tasks to complete before shutting down",
                    attributes={"active_tasks": len(self._active_tasks)},
                )
            )
            await asyncio.gather(
                *self._active_tasks
            )  # Wait for all active tasks to complete
        except asyncio.CancelledError:
            pass

    # Graceful Shutdown

    def issue_shutdown(self):
        """Shutdown the worker application."""
        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Shutdown signal received!",
            )
        )
        self._shutdown_event.set()

    async def wait_for_shutdown(self):
        """Wait for the worker application to shut down."""
        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Waiting for shutdown to complete...",
            )
        )
        await self._shutdown_complete_event.wait()

    async def _cleanup(self):
        """Cleanup the worker application.

        This method is called when the worker is shutting down. Used for cleaning internal state.
        """
        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Cleaning up",
            )
        )
        if self._broker_client is not None:
            await self._broker_client.disconnect()

        logger.info(
            _(
                message="Cleanup complete",
            )
        )

        self._shutdown_complete_event.set()
