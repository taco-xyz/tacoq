from abc import ABC, abstractmethod


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
