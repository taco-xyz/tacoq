"""Configuration for the broker.

This configuration is used to connect to the broker and declare the necessary
queues and exchanges.
"""

from pydantic import BaseModel


class BrokerConfig(BaseModel):
    """Configuration for a RabbitMQ broker.

    ### Attributes:
    - url: The URL of the broker.
    - test_mode: Whether the worker is running in a test environment.

    ### Usage
    ```python
    broker_config = BrokerConfig(url="amqp://localhost:5672")
    ```
    """

    url: str
    """ The URL of the broker. """

    test_mode: bool = False
    """ Whether the worker is running in a test environment. If it is, certain 
    dangerous operations are allowed, such as deleting all tasks in the queue. """
