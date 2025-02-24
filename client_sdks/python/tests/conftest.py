from src.manager import ManagerClient, ManagerConfig
from src.worker import WorkerApplication, WorkerApplicationConfig
from src.broker import BrokerConfig
import pytest

import os

MANAGER_TEST_URL = os.environ.get("MANAGER_TEST_URL", "http://localhost:3000")
BROKER_TEST_URL = os.environ.get(
    "BROKER_TEST_URL", "amqp://user:password@localhost:5672/"
)

WORKER_KIND_NAME = "test_worker_kind"
WORKER_NAME = "test_worker"

pytest_plugins = ["pytest_asyncio"]


## ==============================
## Manager Fixtures
## ==============================


@pytest.fixture
async def manager_config() -> ManagerConfig:
    """Fixture that provides a configured ManagerConfig instance."""
    return ManagerConfig(url=MANAGER_TEST_URL)


@pytest.fixture
def manager_client(manager_config: ManagerConfig) -> ManagerClient:
    return ManagerClient(config=manager_config)


@pytest.fixture
def mock_manager_client() -> ManagerClient:
    return ManagerClient(config=ManagerConfig(url="http://test"))


## ==============================
## Broker Fixtures
## ==============================


@pytest.fixture
async def broker_config() -> BrokerConfig:
    """Fixture that provides a configured BrokerConfig instance."""
    return BrokerConfig(url=BROKER_TEST_URL, prefetch_count=10)


## ==============================
## Worker Fixtures
## ==============================


@pytest.fixture
async def worker_config(
    manager_config: ManagerConfig, broker_config: BrokerConfig
) -> WorkerApplicationConfig:
    """Fixture that provides a configured WorkerConfig instance."""
    return WorkerApplicationConfig(
        kind=WORKER_KIND_NAME,
        name=WORKER_NAME,
        manager_config=manager_config,
        broker_config=broker_config,
    )


@pytest.fixture
async def worker_application(
    worker_config: WorkerApplicationConfig,
) -> WorkerApplication:
    """Fixture that provides a configured WorkerClient instance."""
    return WorkerApplication(config=worker_config)
