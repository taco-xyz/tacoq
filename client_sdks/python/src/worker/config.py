from dataclasses import dataclass

from broker.config import BrokerConfig
from manager.config import ManagerConfig


@dataclass
class WorkerApplicationConfig:
    """Configuration for a worker application. This is passed in on
    initialization of the `WorkerApplication` class, and can come from a config
    file or other sources."""

    name: str
    """ The name of the worker. This is used to identify the worker in the manager."""

    kind: str
    """ The kind of worker. This dictates which tasks get routed to this worker."""

    broker_config: BrokerConfig
    """ Configuration for the broker. """

    manager_config: ManagerConfig
    """ Configuration for the manager. """

    workers: int = 1
    """ The number of worker processes to spawn. This is used to scale the worker horizontally."""
