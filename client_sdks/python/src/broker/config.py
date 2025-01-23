from pydantic import BaseModel


class BrokerConfig(BaseModel):
    """Configuration for a RabbitMQ broker."""

    url: str
    """ The URL of the broker. """
