from abc import ABC, abstractmethod
from typing import Any, AsyncGenerator
from uuid import UUID

from models.task import TaskInput


## =========================================
## Base Broker Client
## - Used for both publishers and consumers.
## =========================================


class BrokerClient(ABC):
    """Base class for all broker clients."""

    @abstractmethod
    async def connect(self) -> None:
        """Connects to the broker."""
        pass

    @abstractmethod
    async def disconnect(self) -> None:
        """Disconnects from the broker."""
        pass


## =========================================
## Broker Consumer Client
## - Used only by the workers to consume tasks
## - Used by the manager to consume results
## =========================================


class BrokerConsumerClient(BrokerClient):
    """A client that listens to the broker and consumes from it."""

    @abstractmethod
    def listen(self) -> AsyncGenerator[tuple[TaskInput, UUID, str], None]:
        """Listen to the worker queue.

        ### Example
        ```py
        async for task, task_id, task_kind in broker_consumer_client.listen():
            # Do something with the task
        ```
        """
        pass


## =========================================
## Broker Publisher Client
## - Used by the publisher to publish tasks
## - Used by the worker to publish results
## =========================================


class BrokerPublisherClient(BrokerClient):
    """A client that publishes tasks to the broker."""

    @abstractmethod
    def publish(self, message: Any) -> None:
        """Publish a message to the broker.

        ### Arguments
        - `message`: The message to publish.
        """
        pass
