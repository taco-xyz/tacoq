import os
import pytest
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import (
    BatchSpanProcessor,
)
from opentelemetry.sdk.resources import Resource
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.trace.id_generator import RandomIdGenerator


from src.manager import ManagerClient, ManagerConfig
from src.worker import WorkerApplication, WorkerApplicationConfig
from src.broker import BrokerConfig


MANAGER_TEST_URL = os.environ.get("MANAGER_TEST_URL", "http://localhost:3000")
BROKER_TEST_URL = os.environ.get(
    "BROKER_TEST_URL", "amqp://user:password@localhost:5672/"
)

WORKER_KIND_NAME = "test_worker_kind"
WORKER_NAME = "test_worker"

pytest_plugins = ["pytest_asyncio"]


@pytest.fixture(autouse=True)
def init_tracer():
    """Initialize a test tracer that prints to console."""
    provider = TracerProvider(
        resource=Resource.create(
            {"service.name": "test_python_client", "environment": "test"}
        ),
        id_generator=RandomIdGenerator(),
    )

    # Use Console exporter for immediate visibility in tests
    batch_processor = BatchSpanProcessor(
        OTLPSpanExporter(endpoint="http://localhost:4318/v1/traces")
    )
    provider.add_span_processor(batch_processor)
    # provider.add_span_processor(SimpleSpanProcessor(ConsoleSpanExporter()))

    # Set as global provider
    trace.set_tracer_provider(provider)

    yield provider

    # Shutdown the provider after test
    batch_processor.force_flush()
    provider.shutdown()


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
    return BrokerConfig(url=BROKER_TEST_URL)


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
        broker_prefetch_count=10,
    )


@pytest.fixture
async def worker_application(
    worker_config: WorkerApplicationConfig,
) -> WorkerApplication:
    """Fixture that provides a configured WorkerClient instance."""
    return WorkerApplication(config=worker_config)
