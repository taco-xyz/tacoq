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

    broker_prefetch_count: int
    """ The number of tasks to prefetch from the broker. This also dictates how many asynchronous tasks
     can be executed at once. """

    manager_config: ManagerConfig
    """ Configuration for the manager. """
