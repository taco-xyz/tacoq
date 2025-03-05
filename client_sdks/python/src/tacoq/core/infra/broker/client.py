"""Abstraction on top of RabbitMQ to publish and consume tasks.

This class is not meant to be used directly by the user. Instead, they should
refer to the `PublisherClient` and `WorkerClient` to publish tasks and consume
results, respectively.
"""

import json
from typing import AsyncGenerator, Optional, Self
from aio_pika import Message, connect_robust
from pydantic import BaseModel

from aio_pika.abc import (
    AbstractChannel,
    AbstractQueue,
    AbstractRobustConnection,
    AbstractExchange,
    AbstractIncomingMessage,
)

from tacoq.core.infra.broker.config import BrokerConfig
from tacoq.core.models.task import Task

# =========================================
# Constants
# NOTE: These are super duper important and
# must be consistent across all nodes.
# =========================================

TASK_EXCHANGE = "task_exchange"
""" Single exchange for all task-related messages. """

RELAY_QUEUE = "relay_queue"
""" Queue for relay to receive ALL tasks. """

RELAY_ROUTING_KEY = "#"  # Wildcard to receive all messages
""" Relay receives all messages. """

WORKER_ROUTING_KEY = "tasks.{worker_kind}"
""" Workers only receive tasks for their kind. """


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
    """Base broker client that handles the connection and disconnection to the
    broker.

    ### Attributes:
    - config: The configuration for the broker.
    """

    config: BrokerConfig
    """ Configuration for the broker. """

    _connection: Optional[AbstractRobustConnection] = None
    """ The connection to the RabbitMQ server. """

    _channel: Optional[AbstractChannel] = None
    """ The channel to the RabbitMQ server. """

    _task_exchange: Optional[AbstractExchange] = None
    """ The exchange for task assignments. """

    async def connect(self: Self) -> None:
        """Establish connection to RabbitMQ server and setup channel."""

        self._connection = await connect_robust(self.config.url)
        self._channel = await self._connection.channel()

        # All clients use the same exchange
        self._task_exchange = await self._channel.declare_exchange(
            TASK_EXCHANGE,
            type="topic",  # Topic exchange for routing by worker kind
            durable=True,
        )

        # Declare relay queue - all clients ensure it exists
        relay_queue = await self._channel.declare_queue(
            RELAY_QUEUE,
            durable=True,
            arguments={"x-max-priority": 255},
        )

        await relay_queue.bind(self._task_exchange, routing_key=RELAY_ROUTING_KEY)

    async def disconnect(self: Self) -> None:
        """Close the RabbitMQ connection.

        ### Raises
        - NotConnectedError: If connection is not established
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
    """RabbitMQ client for publishing tasks to workers. Builds on top of the
    base broker client and adds methods for publishing tasks, declaring new
    queues for workers, and purging worker queues."""

    _binded_worker_queues: set[str] = set()
    """ Track which worker queues we've already declared so we don't need
    to declare them again. """

    async def _declare_worker_queue(self: Self, worker_kind: str) -> None:
        """Declare a worker queue if it doesn't exist yet."""
        if worker_kind in self._binded_worker_queues:
            return

        if not self._channel:
            raise RuntimeError("Channel not initialized")

        if not self._task_exchange:
            raise ExchangeNotDeclaredError(
                "Tried to declare worker queue, but exchange was not declared."
            )

        # Create queue for this worker kind
        worker_queue = await self._channel.declare_queue(
            worker_kind,
            durable=True,  # Survive broker restarts
            arguments={"x-max-priority": 255},
        )
        await worker_queue.bind(
            self._task_exchange,
            routing_key=WORKER_ROUTING_KEY.format(worker_kind=worker_kind),
        )

        # TODO: Make this a testing option instead of a comment
        # Create clone queue for debugging
        clone_queue = await self._channel.declare_queue(
            f"{worker_kind}_cloned",
            durable=True,
            arguments={"x-max-priority": 255},
        )
        await clone_queue.bind(
            self._task_exchange,
            routing_key=WORKER_ROUTING_KEY.format(worker_kind=worker_kind),
        )

        self._binded_worker_queues.add(worker_kind)

    async def purge_worker_queue(self: Self, worker_kind: str) -> None:
        """Purges the queue for a worker kind. Only works in test mode."""
        if not self.config.test_mode:
            raise RuntimeError(
                "Flushing queue is only allowed in test mode. Are you sure you want to call this function? Set broker_config.test_mode=True in your broker config."
            )

        if not self._channel:
            raise RuntimeError("Channel not initialized")

        if not self._task_exchange:
            raise ExchangeNotDeclaredError(
                "Tried to flush worker queue, but exchange was not declared."
            )

        # Get the queue
        queue = await self._channel.declare_queue(
            worker_kind,
            durable=True,
            arguments={"x-max-priority": 255},
        )

        # Purge the queue
        await queue.purge()

    async def publish_task(self: Self, task: Task) -> None:
        """Publish a task. The relay will receive it and workers of the correct kind will too."""

        if not self._task_exchange:
            await self.connect()
        if not self._task_exchange:
            raise ExchangeNotDeclaredError(
                "Tried to publish task, but exchange was not declared."
            )

        # Ensure worker queue exists
        await self._declare_worker_queue(task.worker_kind)

        message = Message(body=task.model_dump_json().encode(), priority=task.priority)
        routing_key = WORKER_ROUTING_KEY.format(worker_kind=task.worker_kind)

        await self._task_exchange.publish(message, routing_key=routing_key)


## =========================================
## Worker Client
## =========================================


class WorkerBrokerClient(BaseBrokerClient):
    """RabbitMQ client for workers to consume tasks and publish results.

    ### Attributes:
    - worker_kind: The name of the worker kind.
    - prefetch_count: The number of tasks to prefetch from the broker.
    """

    worker_kind: str
    """ The name of the worker kind. """

    prefetch_count: int
    """ The number of tasks to prefetch from the broker. """

    _queue: Optional[AbstractQueue] = None
    """ Queue for task assignments. """

    async def connect(self: Self) -> None:
        """Establishes a connection to the broker (as does the base class), but
        then declares a queue for the worker kind and binds it to the task
        exchange."""

        await super().connect()
        if not self._channel:
            raise RuntimeError("Channel not initialized")
        if not self._task_exchange:
            raise ExchangeNotDeclaredError(
                "Tried to declare worker queue, but exchange was not declared."
            )

        await self._channel.set_qos(prefetch_count=self.prefetch_count, global_=True)

        # Worker's queue - named after its kind
        routing_key = WORKER_ROUTING_KEY.format(worker_kind=self.worker_kind)
        self._queue = await self._channel.declare_queue(
            self.worker_kind,
            durable=True,
            arguments={"x-max-priority": 255},
        )
        await self._queue.bind(self._task_exchange, routing_key=routing_key)

    async def listen(
        self: Self,
    ) -> AsyncGenerator[tuple[Task, AbstractIncomingMessage], None]:
        """Listen for tasks for this worker's kind. Only acknowledges tasks
        after they are processed.

        ### Yields:
        tuple[Task, AbstractIncomingMessage]: A tuple of the task and the message to be ACK'd or NACK'd.
        """
        if not self._queue:
            raise RuntimeError("Queue not initialized")

        async with self._queue.iterator(no_ack=False) as queue_iter:
            async for message in queue_iter:
                task = Task(**json.loads(message.body))
                yield (task, message)

    async def publish_task_result(self: Self, task: Task) -> None:
        """Publish a task result to the shared results queue.

        ### Arguments:
        - task: The task to publish the result of.
        """

        # Check if the task has a result attached

        if task.output_data is None:
            raise ValueError(
                "Tried to publish task result, but task has no result attached. How did it get to this point?"
            )

        if self._task_exchange is None:
            raise ExchangeNotDeclaredError(
                "Tried to publish task result, but exchange was not declared."
            )

        message = Message(body=task.model_dump_json().encode())

        await self._task_exchange.publish(message, routing_key=TASK_EXCHANGE)
