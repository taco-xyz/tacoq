from pydantic import BaseModel


class BrokerConfig(BaseModel):
    """Configuration for a RabbitMQ broker."""

    url: str
    """ The URL of the broker. """

    test_mode: bool = False
    """ Whether the worker is running in a test environment. If it is, certain dangerous
    operations are allowed, such as deleting all tasks in the queue. """
