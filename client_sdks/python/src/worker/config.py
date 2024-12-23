from dataclasses import dataclass

from broker.config import BrokerConfig
from manager.config import ManagerConfig


@dataclass
class WorkerApplicationConfig:
    """Configuration for a worker application. This is passed in on
    initialization of the `WorkerApplication` class, and can come from a config
    file or other sources.

    ### Attributes
    - `name`: The name of the worker.
    - `broker_config`: Configuration for the broker.
    - `manager_config`: Configuration for the manager.
    """

    name: str
    broker_config: BrokerConfig
    manager_config: ManagerConfig
