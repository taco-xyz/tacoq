from pydantic import BaseModel


class BrokerConfig(BaseModel):
    """Configuration for a broker."""

    url: str
    """ The URL of the broker. """
