import json
from typing import AsyncGenerator, Optional
from uuid import UUID
from broker.config import BrokerConfig
from broker.core import BrokerClient
from aio_pika import Message, connect_robust
from models.task import Task, TaskResult
from pydantic import BaseModel

from aio_pika.abc import (
    AbstractChannel,
    AbstractQueue,
    AbstractRobustConnection,
    AbstractExchange,
)

# =========================================
# Constants - Exchange and queue names
# =========================================

TASK_ASSIGNMENT_EXCHANGE = "task_assignment_worker_{worker_kind}_exchange"
""" Exchange for task assignments (direct type). """


TASK_ASSIGNMENT_MANAGER_QUEUE = "task_assignment_manager_queue"
""" Queue for task assignments for the manager. """

TASK_ASSIGNMENT_WORKER_QUEUE = "task_assignment_worker_{worker_kind}_queue"
""" Queue for task assignments for a worker kind. """

# NOTE: The routing key of a task assignment depends on the worker kind
# because there is one queue per worker kind, shared by all workers of
# that kind.

TASK_RESULT_EXCHANGE = "task_result_exchange"
""" Exchange for task results. """

TASK_RESULT_QUEUE = "task_result_queue"
""" Queue for task results. """

TASK_RESULT_ROUTING_KEY = "task_result_routing_key"
""" Routing key for task results. """


# =========================================
# Errors
# =========================================


class NoChannelError(Exception):
    """Raised when a RabbitMQ client is not connected to the broker while
    trying to perform an operation that requires a channel."""

    pass


class NotConnectedError(Exception):
    """Raised when a RabbitMQ client is not connected to the broker while
    trying to perform an operation that requires a connection."""

    pass


class QueueNotDeclaredError(Exception):
    """Raised when a RabbitMQ client tries to use a queue that has not been
    declared."""

    pass


class ExchangeNotDeclaredError(Exception):
    """Raised when a RabbitMQ client tries to use an exchange that has not been
    declared."""

    pass


## =========================================
## Base Client
## Used by all RabbitMQ Clients.
## Contains basic logic for connecting and
## disconnecting from RabbitMQ.
## =========================================


class BaseClient(BaseModel, BrokerClient):
    """RabbitMQ implementation of the broker interface."""

    config: BrokerConfig
    """ Configuration for the broker. """

    _connection: Optional[AbstractRobustConnection] = None
    """ The connection to the RabbitMQ server. """

    _channel: Optional[AbstractChannel] = None
    """ The channel to the RabbitMQ server. """

    async def connect(self) -> None:
        """Establish connection to RabbitMQ server and setup channel.

        ### Raises
        - `ConnectionError`: If connection to RabbitMQ fails
        """

        self._connection = await connect_robust(self.config.url)
        self._channel = await self._connection.channel()

    async def disconnect(self) -> None:
        """Close RabbitMQ connection.

        ### Raises
        - `RabbitMQNotConnectedError`: If connection is not established
        """

        if self._connection is None:
            raise NotConnectedError(
                "Tried to disconnect from RabbitMQ, but connection was not established."
            )

        # Remove the exchanges
        await self._connection.close()


## =========================================
## Publisher Client
## Used by the publisher to publish tasks.
## - Tasks are published to a fanout exchange
##   that sends to both the manager queue and
##   the correct worker kind queue.
## =========================================


class PublisherClient(BaseClient):
    """RabbitMQ client for publishing tasks to workers.
    Uses a fanout exchange to send tasks to both the manager queue
    and the appropriate worker kind queue."""

    _worker_kind: str
    """ The kind of worker this publisher targets. """

    _worker_task_queue: Optional[AbstractQueue] = None
    """ The queue for the target worker kind. """

    _manager_task_queue: Optional[AbstractQueue] = None
    """ The queue for the manager to track tasks. """

    _exchange: Optional[AbstractExchange] = None
    """ The fanout exchange for task assignments. """

    async def connect(self) -> None:
        await super().connect()

        if self._channel is None:
            raise NoChannelError(
                "Tried to connect to RabbitMQ, but channel was not established."
            )

        # Create the fanout exchange for task assignments
        exchange = await self._channel.declare_exchange(
            TASK_ASSIGNMENT_EXCHANGE,
            type="fanout",
            durable=self.config.durable,
            auto_delete=self.config.auto_delete,
        )

        # Create queues for both manager and worker kind
        self._manager_task_queue = await self._channel.declare_queue(
            TASK_ASSIGNMENT_MANAGER_QUEUE,
            durable=self.config.durable,
            auto_delete=self.config.auto_delete,
        )
        await self._manager_task_queue.bind(exchange)

        self._worker_task_queue = await self._channel.declare_queue(
            TASK_ASSIGNMENT_WORKER_QUEUE.format(worker_kind=self._worker_kind),
            durable=self.config.durable,
            auto_delete=self.config.auto_delete,
        )
        await self._worker_task_queue.bind(exchange)

    async def publish_task(self, task: Task) -> None:
        """Publish a task to both manager and worker queues via fanout exchange."""
        if self._exchange is None:
            raise ExchangeNotDeclaredError(
                "Tried to publish task, but exchange was not declared."
            )

        message = Message(
            body=task.model_dump_json().encode(),
            headers={"task_id": str(task.id)},
        )

        await self._exchange.publish(message, "")  # Empty routing key for fanout


## =========================================
## Worker Client
## Used by workers to:
##   1. Listen for task assignments from their
##      specific worker kind queue.
##   2. Publish task results to the shared
##      results queue.
## =========================================


class WorkerClient(BaseClient):
    """RabbitMQ client for workers to consume tasks and publish results.
    Each worker kind has its own queue for task assignments, but all workers
    share a single queue for publishing results."""

    _worker_kind: str
    """ The kind of worker. """

    _worker_task_queue: Optional[AbstractQueue] = None
    """ Queue for receiving tasks (specific to worker kind). """

    _result_queue: Optional[AbstractQueue] = None
    """ Queue for publishing results (shared by all workers). """

    _result_exchange: Optional[AbstractExchange] = None
    """ Exchange for task results. """

    def __init__(self, config: BrokerConfig, worker_kind: str):
        super().__init__(config=config)
        self._worker_kind = worker_kind

    async def connect(self) -> None:
        await super().connect()

        if self._channel is None:
            raise NoChannelError(
                "Tried to connect to RabbitMQ, but channel was not established."
            )

        # Setup task assignment queue for this worker kind
        self._worker_task_queue = await self._channel.declare_queue(
            TASK_ASSIGNMENT_WORKER_QUEUE.format(worker_kind=self._worker_kind),
            durable=self.config.durable,
            auto_delete=self.config.auto_delete,
        )

        # Setup result publishing infrastructure
        self._result_queue = await self._channel.declare_queue(
            TASK_RESULT_QUEUE,
            durable=self.config.durable,
            auto_delete=self.config.auto_delete,
        )

        self._result_exchange = await self._channel.declare_exchange(
            TASK_RESULT_EXCHANGE,
            type="direct",
            durable=self.config.durable,
            auto_delete=self.config.auto_delete,
        )

    async def listen(self) -> AsyncGenerator[Task, None]:
        """Listen for tasks assigned to this worker's kind."""
        if self._worker_task_queue is None:
            raise QueueNotDeclaredError(
                "Tried to listen for tasks, but queue was not declared."
            )

        async for message in self._worker_task_queue.iterator():
            async with message.process():
                yield Task(**json.loads(message.body.decode()))

    async def publish_task_result(self, task_id: UUID, task_result: TaskResult) -> None:
        """Publish a task result to the shared results queue."""
        if self._result_exchange is None:
            raise ExchangeNotDeclaredError(
                "Tried to publish task result, but exchange was not declared."
            )

        message = Message(
            body=task_result.model_dump_json().encode(),
            headers={"task_id": str(task_id)},
        )

        await self._result_exchange.publish(message, TASK_RESULT_ROUTING_KEY)
