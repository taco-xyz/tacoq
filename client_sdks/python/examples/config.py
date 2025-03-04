from core.infra.relay.config import RelayConfig
from core.infra.broker.config import BrokerConfig
from worker.client import WorkerApplicationConfig

# Manager and Broker configurations
relay_config = RelayConfig(url="http://localhost:3000")
broker_config = BrokerConfig(url="amqp://user:password@localhost:5672")

# Worker configuration
worker_config = WorkerApplicationConfig(
    name="test_worker",
    kind="test_worker",
    relay_config=relay_config,
    broker_config=broker_config,
    broker_prefetch_count=5,
)

# Task name constants
TASK_1_NAME = "task_1"
TASK_2_NAME = "task_2"
