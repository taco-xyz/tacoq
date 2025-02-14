import json
from typing import AsyncGenerator, Optional
from broker.config import BrokerConfig
from aio_pika import Message, connect_robust
from models.task import Task
from pydantic import BaseModel
from logging import warning
from base64 import b64encode

from aio_pika.abc import (
    AbstractChannel,
    AbstractQueue,
    AbstractRobustConnection,
    AbstractExchange,
)

# =========================================
# Constants
# =========================================

TASK_EXCHANGE = "task_exchange"
""" Single exchange for all task-related messages. """

MANAGER_QUEUE = "manager_queue"
""" Queue for manager to receive ALL tasks. """

MANAGER_ROUTING_KEY = "#"  # Wildcard to receive all messages
""" Manager receives all messages. """

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
    """RabbitMQ implementation of the broker interface."""

    config: BrokerConfig
    """ Configuration for the broker. """

    _connection: Optional[AbstractRobustConnection] = None
    """ The connection to the RabbitMQ server. """

    _channel: Optional[AbstractChannel] = None
    """ The channel to the RabbitMQ server. """

    _task_exchange: Optional[AbstractExchange] = None
    """ The exchange for task assignments. """

    async def connect(self) -> None:
        """Establish connection to RabbitMQ server and setup channel."""

        self._connection = await connect_robust(self.config.url)
        self._channel = await self._connection.channel()

        # All clients use the same exchange
        self._task_exchange = await self._channel.declare_exchange(
            TASK_EXCHANGE,
            type="topic",  # Topic exchange for routing by worker kind
            durable=True,
        )

        # Declare manager queue - all clients ensure it exists
        manager_queue = await self._channel.declare_queue(MANAGER_QUEUE, durable=True)
        await manager_queue.bind(self._task_exchange, routing_key=MANAGER_ROUTING_KEY)

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
    """RabbitMQ client for publishing tasks to workers."""

    _binded_worker_queues: set[str] = set()
    """ Track which worker queues we've already declared. """

    async def _declare_worker_queue(self, worker_kind: str) -> None:
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
        )
        await worker_queue.bind(
            self._task_exchange,
            routing_key=WORKER_ROUTING_KEY.format(worker_kind=worker_kind),
        )
        self._binded_worker_queues.add(worker_kind)

    async def publish_task(self, task: Task) -> None:
        """Publish a task. The manager will receive it and workers of the correct kind will too."""

        if not self._task_exchange:
            await self.connect()
        if not self._task_exchange:
            raise ExchangeNotDeclaredError(
                "Tried to publish task, but exchange was not declared."
            )

        # Ensure worker queue exists
        await self._declare_worker_queue(task.worker_kind)

        message = Message(body=task.model_dump_json().encode())
        routing_key = WORKER_ROUTING_KEY.format(worker_kind=task.worker_kind)

        await self._task_exchange.publish(message, routing_key=routing_key)


## =========================================
## Worker Client
## =========================================


class WorkerBrokerClient(BaseBrokerClient):
    """RabbitMQ client for workers to consume tasks and publish results.
    Each worker kind has its own queue for task assignments, but all workers
    share a single queue for publishing results."""

    worker_kind: str
    """ The name of the worker kind. """

    _queue: Optional[AbstractQueue] = None
    """ Queue for task assignments. """

    async def connect(self) -> None:
        await super().connect()
        if not self._channel:
            raise RuntimeError("Channel not initialized")
        if not self._task_exchange:
            raise ExchangeNotDeclaredError(
                "Tried to declare worker queue, but exchange was not declared."
            )

        # Set prefetch to 1 for fair dispatch
        await self._channel.set_qos(prefetch_count=1)

        # Worker's queue - named after its kind
        routing_key = WORKER_ROUTING_KEY.format(worker_kind=self.worker_kind)
        self._queue = await self._channel.declare_queue(self.worker_kind, durable=True)
        await self._queue.bind(self._task_exchange, routing_key=routing_key)

    async def listen(self) -> AsyncGenerator[Task, None]:
        """Listen for tasks for this worker's kind. Only acknowledges tasks after they are processed."""
        if not self._queue:
            raise RuntimeError("Queue not initialized")

        async with self._queue.iterator() as queue_iter:
            async for message in queue_iter:
                task = Task(**json.loads(message.body.decode()))
                try:
                    yield task
                    await message.ack()  # After the task is processed, acknowledge it
                except Exception as e:
                    await message.reject(requeue=True)
                    warning(f"Failed to process task {task.id}: {e}")

    async def publish_task_result(self, task: Task) -> None:
        """Publish a task result to the shared results queue.

        ### Args:
            task: The task to publish the result of.
        """

        # Check if the task has a result attached

        if task.result is None:
            raise ValueError(
                "Tried to publish task result, but task has no result attached. How did it get to this point?"
            )

        if self._task_exchange is None:
            raise ExchangeNotDeclaredError(
                "Tried to publish task result, but exchange was not declared."
            )

        if isinstance(task.input_data, dict):
            task.input_data = json.dumps(task.input_data)
        task.result = json.dumps(task.result.model_dump_json())

        message = Message(body=task.model_dump_json().encode())

        await self._task_exchange.publish(message, routing_key=TASK_EXCHANGE)
