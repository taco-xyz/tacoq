from pydantic import BaseModel


class BrokerConfig(BaseModel):
    """Configuration for a RabbitMQ broker."""

    url: str
    """ The URL of the broker. """

    prefetch_count: int
    """ The number of tasks to prefetch from the broker. This also dictates how many asynchronous tasks
     can be executed at once. """

    test_mode: bool = False
    """ Whether the worker is running in a test environment. If it is, certain dangerous
    operations are allowed, such as deleting all tasks in the queue. """
