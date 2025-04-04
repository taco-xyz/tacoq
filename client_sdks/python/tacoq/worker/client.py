"""Worker client for the TacoQ client SDK.

This client is used to listen for tasks from the broker, execute them, and
publish the results back to the relay.
"""

import asyncio
import inspect
import json
from datetime import datetime
from typing import (
    Any,
    Awaitable,
    Callable,
    Dict,
    Generic,
    Optional,
    TypeVar,
    get_type_hints,
)

from aio_pika.abc import (
    AbstractIncomingMessage,
)
from opentelemetry.propagate import extract
from opentelemetry.trace import Status, StatusCode
from pydantic import BaseModel
from tenacity import retry, wait_exponential
from typing_extensions import Self

from tacoq.core.encoding import (
    create_decoder,
    create_encoder,
)
from tacoq.core.encoding import Decoder, Encoder
from tacoq.core.infra.broker import WorkerBrokerClient
from tacoq.core.models import (
    SerializedException,
    TaskAssignmentUpdate,
    TaskCompletedUpdate,
    TaskRawInput,
    TaskRawOutput,
    TaskRunningUpdate,
)
from tacoq.core.telemetry import LoggerManager, TracerManager
from tacoq.core.telemetry import StructuredMessage as _
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
        registered_tasks: list[str],
    ):
        self.message = f"Task {task_kind} not registered for this worker. Available tasks: {registered_tasks}"
        super().__init__(self.message)


# =========================================
# Worker Application
# =========================================

TaskInputType = TypeVar("TaskInputType")
TaskOutputType = TypeVar("TaskOutputType")

TaskHandlerFunction = Callable[[TaskInputType], Awaitable[TaskOutputType]]


class TaskHandler(BaseModel, Generic[TaskInputType, TaskOutputType]):
    """A task handler for a specific task kind.

    ### Attributes:
    - kind: The name of the task kind that uniquely identifies this task
      within the worker.
    - task_function: The function that will be called to execute the task.
    - input_decoder: The decoder that will be used to decode the task input
      from bytes to the input type after receiving it from the broker.
    - output_encoder: The encoder that will be used to encode the task output
      into bytes before queing it into the broker.
    """

    kind: str
    task_function: TaskHandlerFunction[TaskInputType, TaskOutputType]
    input_decoder: Decoder[TaskInputType]
    output_encoder: Encoder[TaskOutputType]

    # We need this to allow generics in the model
    model_config = {"arbitrary_types_allowed": True}

    async def execute(self, input_data: TaskRawInput) -> TaskRawOutput:
        """Execute a task based on the raw input data, returning the raw output
        data. Also takes care of the conversion between the input and output
        types and the encoding/decoding of the data.

        ### Arguments
        - input_data: The input data for the task.

        ### Returns
        - The output data from the task.
        """

        # Decode raw input data
        decoded_task_input: TaskInputType = self.input_decoder.decode(input_data)

        # Execute task, returning the decoded output data
        decoded_task_output: TaskOutputType = await self.task_function(
            decoded_task_input
        )

        # Encode the output data and return it.
        encoded_task_output: TaskRawOutput = self.output_encoder.encode(
            decoded_task_output
        )

        return encoded_task_output


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

    # Create a Pydantic model for the input and output of the task
    class MyInputModel(BaseModel):
        name: str

    class MyOutputModel(BaseModel):
        result: str

    # Register a task. It must be async!
    @worker.task(kind="my_task", input_decoder=PydanticDecoder(model=MyInputModel))
    async def my_task(input_data: MyInputModel) -> MyOutputModel:
        return MyOutputModel(result=f"Hello, {input_data.name}!")

    # Start the worker
    await worker.entrypoint()
    ```
    You can can also initialize the worker within your existing application
    using `asyncio.create_task`:
    ```python
    asyncio.create_task(worker.entrypoint())
    ```
    This makes it so you could, for example, initialize the worker within your
    FastAPI application while keeping it running on a single thread. This is,
    however, only recommended for non-blocking tasks. If your workload is
    at all blocking, make sure to isolate your worker in either a separate
    process or an entirely different application.

    You can also issue a shutdown signal to the worker application using
    `worker.issue_shutdown()`. This will cause the worker to shut down gracefully
    after the existing tasks are finished. You can await its shutdown using
    `await worker.wait_for_shutdown()`.
    """

    config: WorkerApplicationConfig
    """ The configuration for this worker application. """

    _registered_tasks: Dict[str, TaskHandler[Any, Any]] = {}
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

    def __init__(self: Self, config: WorkerApplicationConfig) -> None:
        super().__init__(config=config)
        self._registered_tasks = {}  # This must be set here so that the dictionary is per-instance

    # ================================
    # Task Registration & Execution
    # ================================

    def register_task(
        self: Self,
        kind: str,
        task: TaskHandlerFunction[TaskInputType, TaskOutputType],
        input_decoder: Optional[Decoder[TaskInputType]] = None,
        output_encoder: Optional[Encoder[TaskOutputType]] = None,
    ):
        """Register a task handler function for a specific task kind.

        ### Parameters
        - `kind`: Unique identifier for the task type
        - `task`: Async function that processes tasks of this kind
        - `input_decoder`: Decoder for the input data of the task. When set
          to `None`, type hints will be used to infer the decoding logic. See
          `Behaviour` for more details.
        - `output_encoder`: Encoder for the output data of the task. When set
          to `None`, type hints will be used to infer the encoding logic. See
          `Behaviour` for more details.

        ### Behaviour
        - If you don't specify `input_decoder` or `output_encoder`, type hints
          are used to infer which decoder to use, fabricating one based on the
          types. See more in `tacoq.core.encoding.polymorphic`.
        - Re-registering a task will OVERWRITE the previous task handler - it
          WILL NOT throw an error.

        ### Usage
        ```python

        class MyInputModel(BaseModel):
            name: str

        class MyOutputModel(BaseModel):
            result: str

        async def my_task(input_data: MyInputModel) -> MyOutputModel:
            return MyOutputModel(result=f"Hello, {input_data.name}!")

        worker.register_task(
            kind="my_task",
            task=my_task,
        )
        ```
        """

        # Get type hints so we can infer the input and output types for polymorphic
        # encoding and decoding
        type_hints = get_type_hints(task)
        input_type = None
        output_type = None

        # Validate that either an input decoder or input type is specified
        if input_decoder is None:
            input_type = type_hints.get(
                list(inspect.signature(task).parameters.keys())[0]
            )
            if input_type is None:
                raise ValueError(
                    "Input type is not specified for task with unspecified input decoder. Please specify either of these."
                )
            input_decoder = create_decoder(input_type)

        # Validate that either an output encoder or output type is specified
        if output_encoder is None:
            output_type = type_hints.get("return")
            if output_type is None:
                raise ValueError(
                    "Output type is not specified for task with unspecified output encoder. Please specify either of these."
                )
            output_encoder = create_encoder(output_type)

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

        self._registered_tasks[kind] = TaskHandler(
            kind=kind,
            task_function=task,
            input_decoder=input_decoder,
            output_encoder=output_encoder,
        )

    def task(
        self: Self,
        kind: str,
        input_decoder: Optional[Decoder[TaskInputType]] = None,
        output_encoder: Optional[Encoder[TaskOutputType]] = None,
    ) -> Callable[
        [TaskHandlerFunction[TaskInputType, TaskOutputType]],
        TaskHandlerFunction[TaskInputType, TaskOutputType],
    ]:
        """Decorator for registering task handler functions.

        ### Arguments
        - kind: Unique identifier for the task type
        - input_decoder: Decoder for the input data of the task. When set
          to `None`, type hints will be used to infer the decoding logic. See
          `WorkerApplication.register_task` for more details.
        - output_encoder: Encoder for the output data of the task. When set
          to `None`, type hints will be used to infer the encoding logic. See
          `WorkerApplication.register_task` for more details.

        ### Returns
        - Callable: Decorator function that registers the task handler

        ### Usage
        ```python

        class MyInputModel(BaseModel):
            name: str

        class MyOutputModel(BaseModel):
            result: str

        @worker.task(kind="my_task")
        async def my_task(input_data: MyInputModel) -> MyOutputModel:
            return MyOutputModel(result=f"Hello, {input_data.name}!")
        ```
        """

        def decorator(
            task: TaskHandlerFunction[TaskInputType, TaskOutputType],
        ) -> TaskHandlerFunction[TaskInputType, TaskOutputType]:
            self.register_task(kind, task, input_decoder, output_encoder)
            return task

        return decorator

    async def _execute_task_assignment(
        self: Self,
        task_assignment_update: TaskAssignmentUpdate,
        message: AbstractIncomingMessage,
    ):
        """Execute a task and update its status in the relay.

        ### Arguments
        - task: Task to execute
        - message: Message to acknowledge
        """

        # Check if broker client is initialized
        if self._broker_client is None:
            raise RuntimeError("Broker client not initialized")

        # Extract OTEL context from the task and init loggers and tracers
        otel_ctx_carrier = extract(task_assignment_update.otel_ctx_carrier)

        tracer = TracerManager.get_tracer()
        logger = LoggerManager.get_logger()

        with tracer.start_as_current_span(
            "task_worker_lifecycle",
            context=otel_ctx_carrier,
            attributes={
                "task.id": str(task_assignment_update.id),
                "task.kind": task_assignment_update.task_kind,
                "worker.kind": self.config.kind,
            },
        ) as parent_span:
            # Find task handler. If it doesn't exist, we nack the message and return
            # If this task is not registered anywhere, it will loop infinitely through
            # the workers. That's the user's problem.

            task_handler = self._registered_tasks.get(task_assignment_update.task_kind)
            if task_handler is None:
                error = TaskNotRegisteredError(
                    task_assignment_update.task_kind,
                    list(self._registered_tasks.keys()),
                )
                parent_span.set_status(Status(StatusCode.ERROR))
                parent_span.record_exception(error)
                logger.error(
                    _(
                        message=f"Task of kind {task_assignment_update.task_kind} not registered. Available tasks: {self._registered_tasks.keys()}",
                        attributes={
                            "task.kind": task_assignment_update.task_kind,
                            "available_tasks": list(self._registered_tasks.keys()),
                        },
                    )
                )
                await message.nack()
                return

            # Task Execution ================================
            # TODO - Improve exception serialization

            result: TaskRawOutput = b""
            is_error: bool = False

            # Send task processing event
            started_at = datetime.now()

            # Submit task processing event via broker
            asyncio.create_task(
                self._broker_client.publish_task_running(
                    TaskRunningUpdate(
                        id=task_assignment_update.id,
                        started_at=started_at,
                        executed_by=self.config.name,
                    )
                )
            )

            # Start timer
            parent_span.set_attribute("task.started_at", started_at.isoformat())

            with tracer.start_as_current_span(
                "task_execution",
                attributes={
                    "task.handler": task_handler.kind,
                    "task.input_size": len(str(task_assignment_update.input_data)),
                },
            ) as execution_span:
                try:
                    result = await task_handler.execute(
                        task_assignment_update.input_data
                    )
                    execution_span.set_attribute("task.output_size", len(str(result)))
                except Exception as e:
                    exception = SerializedException.from_exception(e)
                    result = json.dumps(exception.model_dump()).encode("utf-8")

                    is_error = True
                    logger.error(
                        _(
                            message=f"Error executing task of kind {task_assignment_update.task_kind} with ID {task_assignment_update.id}",
                            attributes={
                                "task.id": str(task_assignment_update.id),
                                "task.kind": task_assignment_update.task_kind,
                                "exception_message": exception.message,
                                "exception_type": exception.type,
                            },
                        )
                    )
                    execution_span.set_status(Status(StatusCode.ERROR))
                    execution_span.record_exception(e)

            # Stop timer
            completed_at = datetime.now()

            # Update task
            output_data = result
            is_error = is_error
            completed_at = completed_at

            # Submission =========================================

            # Submit task output via broker
            with tracer.start_as_current_span(
                "publish_task_result",
                attributes={
                    "task.result_size": len(str(result)),
                    "task.id": str(task_assignment_update.id),
                    "task.kind": task_assignment_update.task_kind,
                },
            ):
                await self._broker_client.publish_task_completed(
                    TaskCompletedUpdate(
                        id=task_assignment_update.id,
                        output_data=output_data,
                        is_error=is_error,
                    )
                )

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

    @retry(wait=wait_exponential(multiplier=1, min=1, max=15))
    async def _init_broker_client(self: Self) -> None:
        """Initialize the broker client for this worker.

        This function will always be retried until a connection is successful.
        This is to ensure that the worker will keep trying to connect to the
        broker until it succeeds.
        """

        # Init the broker client using the queue name of the worker kind
        self._broker_client = WorkerBrokerClient(
            config=self.config.broker_config,
            worker_kind=self.config.kind,
            prefetch_count=self.config.broker_prefetch_count,
        )

        try:
            await self._broker_client.connect()
        except Exception as e:
            logger = LoggerManager.get_logger()
            logger.error(
                _(message="Error connecting to broker", attributes={"error": str(e)})
            )
            raise e

    async def entrypoint(self: Self) -> None:
        """Initialize and start listening for tasks."""

        # Initialize the broker client
        await self._init_broker_client()

        logger = LoggerManager.get_logger()
        logger.info(
            _(
                message="Worker application initialized!",
            )
        )

        # Start listening for tasks
        try:
            await self._listen()
        except asyncio.CancelledError:
            pass

        # Clean up after everything is done
        finally:
            logger.info(
                _(
                    message="Worker application shutdown complete",
                )
            )
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
                task_assignment, message = await asyncio.wait_for(
                    self._broker_client.listen().__anext__(),
                    timeout=1.0,  # Check shutdown signal every second
                )

                # Task is created and added to the tracker pool
                logger.info(
                    _(
                        message=f"Received task of kind {task_assignment.task_kind} with ID {task_assignment.id}",
                        attributes={
                            "task.id": str(task_assignment.id),
                            "task.kind": task_assignment.task_kind,
                        },
                    )
                )
                async_task = asyncio.create_task(
                    self._execute_task_assignment(task_assignment, message)
                )
                async_task.add_done_callback(self._active_tasks.discard)
                self._active_tasks.add(async_task)
                async_task.add_done_callback(self._active_tasks.discard)
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                logger.error(
                    _(message="Error listening for tasks", attributes={"error": str(e)})
                )
                raise e

    # Graceful Shutdown

    def issue_shutdown(self: Self) -> None:
        """Issue a shutdown signal to the worker application. This will cause the
        worker to shut down gracefully after the existing tasks are finished.
        You can await its shutdown using `await worker.wait_for_shutdown()`.
        ### Usage:

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
        """Wait for the worker application to shut down. This is non-blocking.

        ### Usage:
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
        logger.info(
            _(
                message=f"Waiting for {len(self._active_tasks)} active tasks to complete before shutting down...",
                attributes={"active_tasks": len(self._active_tasks)},
            )
        )

        await asyncio.gather(*self._active_tasks)

        # Disconnect from broker

        logger.info(
            _(
                message="No active tasks left.Disconnecting from broker",
            )
        )

        try:
            if self._broker_client is not None:
                await self._broker_client.disconnect()
        except Exception as e:
            logger.error(
                _(
                    message="Error disconnecting from broker",
                    attributes={"error": str(e)},
                )
            )

        logger.info(
            _(
                message="Cleanup complete",
            )
        )

        self._shutdown_complete_event.set()
