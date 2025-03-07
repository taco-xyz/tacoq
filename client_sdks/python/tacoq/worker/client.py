"""Worker client for the TacoQ client SDK.

This client is used to listen for tasks from the broker, execute them, and
publish the results back to the relay.
"""

import asyncio
import json
from datetime import datetime
from typing import Awaitable, Callable, Dict, Optional

from aio_pika.abc import (
    AbstractIncomingMessage,
)
from tacoq.core.infra.broker import WorkerBrokerClient
from tacoq.core.models import (
    SerializedException,
    Task,
    TaskInput,
    TaskOutput,
    TaskStatus,
)
from tacoq.core.telemetry import LoggerManager, TracerManager
from tacoq.core.telemetry import StructuredMessage as _
from opentelemetry.propagate import extract
from opentelemetry.trace import Status, StatusCode
from pydantic import BaseModel
from typing_extensions import Self

from tacoq.worker.config import WorkerApplicationConfig

# =========================================
# Errors
# =========================================


class TaskNotRegisteredError(Exception):
    """Exception raised when a task tries to be executed but it hasn't been
    registered in the current worker."""

    def __init__(
        self: Self,
        task_kind: str,
        registered_tasks: Dict[str, Callable[[TaskInput], Awaitable[TaskOutput]]],
    ):
        self.message = f"Task {task_kind} not registered for this worker. Available tasks: {registered_tasks.keys()}"
        super().__init__(self.message)


# =========================================
# Worker Application
# =========================================


class WorkerApplication(BaseModel):
    """A worker application that processes tasks from a task queue.

    ### Attributes:
    - config: The configuration for this worker application. See `WorkerApplicationConfig` for more details.

    ### Usage
    ```python
    # Set up the config
    config = WorkerApplicationConfig(
        kind="my_worker",
        relay_config=RelayConfig(url="http://localhost:8080"),
        broker_config=BrokerConfig(url="amqp://localhost:5672"),
        broker_prefetch_count=10,
    )

    # Initialize the worker with the config
    worker = WorkerApplication(config=config)

    # Register a task. It must be async!
    @worker.task(kind="my_task")
    async def my_task(input_data: TaskInput) -> TaskOutput:
        return TaskOutput(result="Hello, world!")

    # Start the worker
    await worker.entrypoint()
    ```
    You can can also initialize the worker within your existing application
    using `asyncio.create_task`:
    ```python
    asyncio.create_task(worker.entrypoint())
    ```
    This makes it so you could, for example, initialize the worker within your
    FastAPI application while keeping it running on a single thread. However,
    this is only recommended for non-blocking tasks. If your workload is
    at all blocking, make sure to isolate your worker in either a separate
    process or an entirely different application.

    You can also issue a shutdown signal to the worker application using
    `worker.issue_shutdown()`. This will cause the worker to shut down gracefully
    after the existing tasks are finished. You can await its shutdown using
    `await worker.wait_for_shutdown()`.
    """

    config: WorkerApplicationConfig
    """ The configuration for this worker application. """

    _registered_tasks: Dict[str, Callable[[TaskInput], Awaitable[TaskOutput]]] = {}
    """ All the tasks that this worker application can handle. """

    _broker_client: Optional[WorkerBrokerClient] = None
    """ The broker client that this worker application uses. """

    # Graceful Shutdown

    _shutdown_event: asyncio.Event = asyncio.Event()
    """ Event that is set when the worker application is shutting down. """

    _shutdown_complete_event: asyncio.Event = asyncio.Event()
    """ Event that is set when the worker application has completed shutting down. """

    _active_tasks: set[asyncio.Task[None]] = set()
    """ The set of active tasks that this worker application is processing. """

    def model_post_init(self: Self, _) -> None:
        self._registered_tasks = {}

    # ================================
    # Task Registration & Execution
    # ================================

    def register_task(
        self: Self, kind: str, task: Callable[[TaskInput], Awaitable[TaskOutput]]
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
        self: Self, kind: str
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

    async def _execute_task(self: Self, task: Task, message: AbstractIncomingMessage):
        """Execute a task and update its status in the relay.

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

    async def _init_broker_client(self: Self) -> None:
        """Initialize the broker client for this worker.

        ### Raises:
        - RuntimeError: If relay client is not initialized
        """

        # Init the broker client using the queue name of the worker kind
        self._broker_client = WorkerBrokerClient(
            config=self.config.broker_config,
            worker_kind=self.config.kind,
            prefetch_count=self.config.broker_prefetch_count,
        )
        await self._broker_client.connect()

    async def entrypoint(self: Self) -> None:
        """Entrypoint into the worker application."""
        return await self._lifecycle()

    async def _lifecycle(self: Self) -> None:
        """Initialize and start listening for tasks."""

        # Initialize the broker client
        await self._init_broker_client()

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Worker application initialized!",
                attributes={"worker.kind": self.config.kind},
            )
        )

        # Start listening for tasks
        try:
            await self._listen()
        except asyncio.CancelledError:
            pass

        # Clean up after everything is done
        finally:
            await self._cleanup()

    # Loop

    async def _listen(self: Self) -> None:
        """Listen for tasks from the broker, setting them to be executed in the background.

        ### Raises:
        - RuntimeError: If broker client is not initialized
        """

        if self._broker_client is None:
            raise RuntimeError("Broker client not initialized")

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Listening for tasks",
            )
        )
        # Loop
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
                async_task.add_done_callback(self._active_tasks.discard)
                self._active_tasks.add(async_task)
            except asyncio.TimeoutError:
                continue

    # Graceful Shutdown

    def issue_shutdown(self: Self) -> None:
        """Issue a shutdown signal to the worker application.

        ### Usage:
        Issue a shutdown signal to the worker application. This will cause the
        worker to shut down gracefully after the existing tasks are finished.
        You can await its shutdown using `await worker.wait_for_shutdown()`.

        ```python
        worker.issue_shutdown()
        ```
        """

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Shutdown signal received!",
            )
        )
        self._shutdown_event.set()

    async def wait_for_shutdown(self: Self) -> None:
        """Wait for the worker application to shut down.

        ### Usage:
        Wait for the worker application to shut down. This is non-blocking.
        ```python
        worker.issue_shutdown()
        await worker.wait_for_shutdown()
        ```
        """
        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Waiting for shutdown to complete...",
            )
        )
        await self._shutdown_complete_event.wait()

    async def _cleanup(self: Self) -> None:
        """Cleanup the internal state of the worker application."""

        # Wait for all active tasks to complete

        logger = LoggerManager.get_logger()

        # Wait for all active tasks to complete

        logger.info(
            _(
                message=f"Waiting for {len(self._active_tasks)} active tasks to complete before shutting down",
                attributes={"active_tasks": len(self._active_tasks)},
            )
        )

        await asyncio.gather(*self._active_tasks)

        # Disconnect from broker

        logger.info(
            _(
                message="Disconnecting from broker",
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
