from manager import ManagerClient, ManagerConfig, ManagerStates
from worker import WorkerApplication, WorkerApplicationConfig
from broker import BrokerConfig
import pytest

MANAGER_TEST_URL = "http://localhost:3000"
BROKER_TEST_URL = "amqp://user:password@localhost:5672/"

WORKER_NAME = "test_worker"

pytest_plugins = ["pytest_asyncio"]


@pytest.fixture
async def manager_config() -> ManagerConfig:
    """Fixture that provides a configured ManagerConfig instance."""
    return ManagerConfig(url=MANAGER_TEST_URL)


@pytest.fixture
async def manager_client(manager_config: ManagerConfig) -> ManagerClient:
    """Fixture that provides a configured ManagerClient instance. Also checks
    whether the manager client is running and healthy before starting the tests."""

    client = ManagerClient(config=manager_config)

    # Check if the manager is healthy
    client_health = await client.check_health()

    if client_health != ManagerStates.HEALTHY:
        raise RuntimeError(f"Manager is not healthy. Current state: {client_health}")

    return client


@pytest.fixture
async def broker_config() -> BrokerConfig:
    """Fixture that provides a configured BrokerConfig instance."""
    return BrokerConfig(url=BROKER_TEST_URL)


@pytest.fixture
async def worker_config(
    manager_config: ManagerConfig, broker_config: BrokerConfig
) -> WorkerApplicationConfig:
    """Fixture that provides a configured WorkerConfig instance."""
    return WorkerApplicationConfig(
        name=WORKER_NAME, manager_config=manager_config, broker_config=broker_config
    )


@pytest.fixture
async def worker_application(
    worker_config: WorkerApplicationConfig,
) -> WorkerApplication:
    """Fixture that provides a configured WorkerClient instance."""
    return WorkerApplication(config=worker_config)
