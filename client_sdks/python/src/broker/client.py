import asyncio
import json
from typing import AsyncGenerator, Optional
from broker.config import BrokerConfig
from aio_pika import Message, connect_robust
from models.task import Task
from pydantic import BaseModel
from logging import warn

from aio_pika.abc import (
    AbstractChannel,
    AbstractQueue,
    AbstractRobustConnection,
    AbstractExchange,
)

# =========================================
# Constants - Exchange and queue names
# =========================================

# Publisher

TASK_ASSIGNMENT_EXCHANGE = "task_assignment_exchange"
""" Exchange for task assignments. Used by the publisher to 
send tasks to the manager and workers. """

TASK_ASSIGNMENT_QUEUE = "task_assignment_queue"
""" Queue for task assignments. Used by the publisher to send 
tasks to the manager and workers. """

MANAGER_ROUTING_KEY = "tasks.#"  # Matches all tasks
""" Routing key for the manager queue. Receives all tasks to 
save them. """

WORKER_ROUTING_KEY_PREFIX = "tasks.{worker_kind}"  # Will be combined with worker_kind
""" Routing key for worker queues. Only workers of a 
specific kind will receive these tasks. """


def get_worker_routing_key(worker_kind: str) -> str:
    """Get the routing key for a worker kind.

    ### Args:
    - `worker_kind`: The kind of worker to get the routing key for.
    """
    return WORKER_ROUTING_KEY_PREFIX.format(worker_kind=worker_kind)


# Worker

TASK_RESULT_EXCHANGE = "task_result_exchange"
""" Exchange for task results. Used by all workers to
publish their results. """

TASK_RESULT_QUEUE = "task_results"
""" Queue for task results. Used by all workers to publish their results. """


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

    _binded_worker_queues: set[str] = set()
    """ The queues that have been binded to the exchange. We keep track of them
    so we don't have to bind them again every time we submit a new task. """

    async def connect(self) -> None:
        await super().connect()

        if self._channel is None:
            raise NoChannelError(
                "Tried to connect to RabbitMQ, but channel was not established."
            )

        # Declare a topic exchange instead of the default direct
        self._task_exchange = await self._channel.declare_exchange(
            TASK_ASSIGNMENT_EXCHANGE, type="topic", durable=True
        )

        # Declare both queues without binding to them
        manager_queue = await self._channel.declare_queue(
            TASK_ASSIGNMENT_QUEUE, durable=True
        )
        await manager_queue.bind(
            TASK_ASSIGNMENT_EXCHANGE, routing_key=MANAGER_ROUTING_KEY
        )

    async def _declare_worker_queue(self, worker_kind: str) -> None:
        """Declare a worker queue and bind it to the exchange."""

        if worker_kind in self._binded_worker_queues:
            return

        if self._channel is None:
            raise NoChannelError(
                "Tried to declare worker queue, but channel was not established."
            )

        worker_queue = await self._channel.declare_queue(worker_kind, durable=True)
        await worker_queue.bind(
            TASK_ASSIGNMENT_EXCHANGE, routing_key=get_worker_routing_key(worker_kind)
        )
        self._binded_worker_queues.add(worker_kind)

    async def publish_task(self, task: Task) -> None:
        """Publish a task to both manager and worker queues via exchange and routing mechanisms.

        ### Arguments
        - `routing_key`: The routing key for the task. This is based on the worker kind. The publisher
        client will know the routing key based on the requests it has made to the manager, who creates
        the queues and binds them to the exchange.

        ### Raises
        - `RuntimeError`: If the exchange was not declared.
        """

        if self._channel is None:
            await self.connect()

        await self._declare_worker_queue(task.worker_kind)

        if self._task_exchange is None:
            raise RuntimeError("Tried to publish task, but exchange was not declared.")

        message = Message(body=task.model_dump_json().encode())

        await self._task_exchange.publish(
            message, routing_key=get_worker_routing_key(task.worker_kind)
        )


## =========================================
## Worker Client
## =========================================


class WorkerBrokerClient(BaseBrokerClient):
    """RabbitMQ client for workers to consume tasks and publish results.
    Each worker kind has its own queue for task assignments, but all workers
    share a single queue for publishing results."""

    worker_kind: str
    """ The name of the worker kind. """

    _task_assignment_queue: Optional[AbstractQueue] = None
    """ Queue for task assignments. """

    _result_exchange: Optional[AbstractExchange] = None
    """ Exchange for publishing results (shared by all workers). """

    async def connect(self) -> None:
        await super().connect()

        if self._channel is None:
            raise NoChannelError(
                "Tried to connect to RabbitMQ, but channel was not established."
            )

        # =========================================
        # Setup task assignment queue for this worker kind
        # =========================================

        # Set prefetch to one to enable fair dispatching
        await self._channel.set_qos(prefetch_count=1)

        while True:
            try:
                self._task_assignment_queue = await self._channel.declare_queue(
                    self.worker_kind,
                    passive=True,  # We only want to connect to the queue if it already exists.
                )
                break
            except Exception as e:
                warn(
                    f"Failed to passively declare task assignment queue: {e}. Retrying in 1 second...\nThis might mean the queue doesn't exist because: 1. no tasks for this worker kind have been published yet, or 2. there is a mismatch between the worker kind named on the publisher vs the worker kind named on the worker."
                )
                await asyncio.sleep(1)

        # =========================================
        # Setup result publishing infrastructure
        # =========================================

        # Exchange
        self._result_exchange = await self._channel.declare_exchange(
            TASK_RESULT_EXCHANGE,
            durable=True,
        )

        # Set up the result queue and bind it to the exchange
        _result_queue = await self._channel.declare_queue(
            TASK_RESULT_QUEUE, durable=True
        )
        await _result_queue.bind(TASK_RESULT_EXCHANGE)

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

        await self._result_exchange.publish(message, routing_key=TASK_RESULT_QUEUE)
