from broker import BrokerConfig
from manager.config import ManagerConfig
from worker import WorkerApplicationConfig

# Manager and Broker configurations
manager_config = ManagerConfig(url="http://localhost:3000")
broker_config = BrokerConfig(url="http://localhost:5672")

# Worker configuration
worker_config = WorkerApplicationConfig(
    name="test_worker",
    manager_config=manager_config,
    broker_config=broker_config,
)

# Task name constants
TASK_1_NAME = "task_1"
TASK_2_NAME = "task_2"
