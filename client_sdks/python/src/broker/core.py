from abc import ABC, abstractmethod
from typing import AsyncGenerator
from uuid import UUID

from models.task import TaskInput


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
    def listen(self) -> AsyncGenerator[tuple[TaskInput, UUID, str], None]:
        """Listen to the worker queue."""
        pass
