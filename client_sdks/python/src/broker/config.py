from pydantic import BaseModel


class BrokerConfig(BaseModel):
    """Configuration for a broker."""

    url: str
    """ The URL of the broker. """

    durable: bool = True
    """ Whether exchanges and queues are saved on disk."""

    auto_delete: bool = False
    """ Whether exchanges and queues are deleted when they are no longer in use."""
