from abc import ABC, abstractmethod
from typing import AsyncGenerator, Tuple


class BrokerClient(ABC):
    @abstractmethod
    async def connect(self) -> None:
        """Connect to the broker."""
        pass

    @abstractmethod
    async def disconnect(self) -> None:
        """Disconnect from the broker."""
        pass

    @abstractmethod
    async def listen(self) -> AsyncGenerator[Tuple[str, str, str], None]:
        """Listen to the worker queue."""
        pass
