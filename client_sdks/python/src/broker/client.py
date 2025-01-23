import json
from typing import AsyncGenerator, Optional
from broker.config import BrokerConfig
from aio_pika import Message, connect_robust
from models.task import Task
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

TASK_ASSIGNMENT_EXCHANGE = "task_assignment_exchange"
""" Exchange for task assignments. """

TASK_RESULT_EXCHANGE = "task_result_exchange"
""" Exchange for task results. """

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
## =========================================


class BaseBrokerClient(BaseModel):
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
## =========================================


class PublisherBrokerClient(BaseBrokerClient):
    """RabbitMQ client for publishing tasks to workers.
    Uses a fanout exchange to send tasks to both the manager queue
    and the appropriate worker kind queue."""

    _task_exchange: Optional[AbstractExchange] = None
    """ The exchange for task assignments. """

    async def connect(self) -> None:
        await super().connect()

        if self._channel is None:
            raise NoChannelError(
                "Tried to connect to RabbitMQ, but channel was not established."
            )

        # Connect to the exchange so to which we will publish tasks.
        self._task_exchange = await self._channel.declare_exchange(
            TASK_ASSIGNMENT_EXCHANGE,
            passive=True,  # We only want to connect to the exchange if it already exists.
        )

    async def publish_task(self, routing_key: str, task: Task) -> None:
        """Publish a task to both manager and worker queues via exchange and routing mechanisms.

        ### Arguments
        - `routing_key`: The routing key for the task. This is based on the worker kind. The publisher
        client will know the routing key based on the requests it has made to the manager, who creates
        the queues and binds them to the exchange.

        ### Raises
        - `RuntimeError`: If the exchange was not declared.
        """

        if self._task_exchange is None:
            raise RuntimeError("Tried to publish task, but exchange was not declared.")

        message = Message(body=task.model_dump_json().encode())

        await self._task_exchange.publish(message, routing_key)


## =========================================
## Worker Client
## =========================================


class WorkerBrokerClient(BaseBrokerClient):
    """RabbitMQ client for workers to consume tasks and publish results.
    Each worker kind has its own queue for task assignments, but all workers
    share a single queue for publishing results."""

    _task_assignment_queue_name: str
    """ The name of the task assignment queue. """

    _task_assignment_queue: Optional[AbstractQueue] = None
    """ Queue for task assignments. """

    _result_exchange: Optional[AbstractExchange] = None
    """ Exchange for publishing results (shared by all workers). """

    def __init__(self, config: BrokerConfig, task_assignment_queue_name: str):
        super().__init__(config=config)
        self._task_assignment_queue_name = task_assignment_queue_name

    async def connect(self) -> None:
        await super().connect()

        if self._channel is None:
            raise NoChannelError(
                "Tried to connect to RabbitMQ, but channel was not established."
            )

        # =========================================
        # Setup task assignment queue for this worker kind
        # =========================================

        self._task_assignment_queue = await self._channel.declare_queue(
            self._task_assignment_queue_name,
            passive=True,  # We only want to connect to the queue if it already exists.
        )

        # =========================================
        # Setup result publishing infrastructure
        # =========================================

        # Set prefetch to one to enable fair dispatching
        await self._channel.set_qos(prefetch_count=1)

        # Setup result publishing infrastructure
        self._result_exchange = await self._channel.declare_exchange(
            TASK_RESULT_EXCHANGE,
            passive=True,  # We only want to connect to the queue if it already exists.
        )

    async def listen(self) -> AsyncGenerator[Task, None]:
        """Listen for tasks assigned to this worker's kind."""
        if self._task_assignment_queue is None:
            raise QueueNotDeclaredError(
                "Tried to listen for tasks, but queue was not declared."
            )

        async for message in self._task_assignment_queue.iterator():
            async with message.process():
                yield Task(**json.loads(message.body.decode()))

    async def publish_task_result(self, task: Task) -> None:
        """Publish a task result to the shared results queue."""

        # Check if the task has a result attached
        if task.result is None:
            raise ValueError(
                "Tried to publish task result, but task has no result attached. How did it get to this point?"
            )

        if self._result_exchange is None:
            raise ExchangeNotDeclaredError(
                "Tried to publish task result, but exchange was not declared."
            )

        message = Message(body=task.model_dump_json().encode())

        await self._result_exchange.publish(message, routing_key="")
