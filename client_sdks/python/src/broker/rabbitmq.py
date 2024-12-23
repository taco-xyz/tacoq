import json
from typing import AsyncGenerator, Tuple
from broker.config import BrokerConfig
from broker.core import BrokerClient
from aio_pika import connect_robust


class RabbitMQBroker(BrokerClient):
    """RabbitMQ implementation of the broker interface.

    Attributes:
        config (BrokerConfig): Configuration for connecting to RabbitMQ
        exchange_name (str): Name of the primary exchange
        worker_id (str): Unique identifier for this worker instance
        connection: Active connection to RabbitMQ server
        channel: Active channel for communication
        exchange: Declared exchange for message routing
    """

    def __init__(self, config: BrokerConfig, exchange_name: str, worker_id: str):
        self.config = config
        self.exchange_name = exchange_name
        self.worker_id = worker_id  # Add worker_id to identify this worker

    async def connect(self) -> None:
        """Establish connection to RabbitMQ server and setup channel.

        ### Raises
        - `ConnectionError`: If connection to RabbitMQ fails
        """
        self.connection = await connect_robust(self.config.url)
        self.channel = await self.connection.channel()

    async def disconnect(self) -> None:
        """Close RabbitMQ connection."""
        # Remove the exchanges
        await self.connection.close()

    async def listen(self) -> AsyncGenerator[Tuple[str, str, str], None]:
        """Listen for tasks of a specific type.

        ### Yields
        - `str`: Decoded message body containing task data

        ### Raises
        - `ConnectionError`: If broker connection is lost
        """
        # The queue should have been created sucessfully on the gateway side
        # The queue name should be the id of the worker
        queue_instance = await self.channel.declare_queue(self.worker_id, durable=False)

        async for message in queue_instance.iterator():
            async with message.process():
                task_kind = message.headers.get("task_kind")
                yield json.loads(message.body.decode()), message.message_id, task_kind

        await queue_instance.delete()
