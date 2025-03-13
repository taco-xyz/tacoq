import os
from time import sleep
from typing import AsyncGenerator
import pytest
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import (
    BatchSpanProcessor,
)
from opentelemetry.sdk.trace.id_generator import RandomIdGenerator
from tacoq.core.infra.broker import BrokerConfig
from tacoq.relay import RelayClient, RelayConfig
from tacoq.core.telemetry import LoggerManager, TracerManager
from tacoq.publisher import PublisherClient
from tacoq.worker import WorkerApplicationConfig

RELAY_TEST_URL = os.environ.get("RELAY_TEST_URL", "http://localhost:3000")
BROKER_TEST_URL = os.environ.get(
    "BROKER_TEST_URL", "amqp://user:password@localhost:5672/"
)

WORKER_KIND_NAME = "test_worker_kind"
WORKER_NAME = "test_worker"

pytest_plugins = ["pytest_asyncio"]


@pytest.fixture(scope="session", autouse=True)
def init_tracer_provider():
    """Initialize a test tracer provider at session level."""

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

    # Set as global provider
    trace.set_tracer_provider(provider)

    # Set up a global logger too
    LoggerManager.get_logger()

    yield provider

    # Shutdown the provider after all tests
    timed_out = batch_processor.force_flush()
    sleep(2)

    if timed_out:
        print("[WARNING]: Failed to flush traces. Some traces may be lost.")

    # Shutdown the provider after test
    provider.shutdown()


@pytest.fixture(autouse=True)
def create_test_span():
    """Create a span for the current test."""
    # Get current test name and init a span
    current_test = (
        os.environ.get("PYTEST_CURRENT_TEST", "").split(":")[-1].split(" ")[0]
    )
    tracer = TracerManager.get_tracer()
    with tracer.start_as_current_span(current_test):
        yield


## ==============================
## Relay Fixtures
## ==============================


@pytest.fixture(scope="session")
def relay_config() -> RelayConfig:
    """Fixture that provides a configured RelayConfig instance."""
    return RelayConfig(url=RELAY_TEST_URL)


@pytest.fixture
async def relay_client(relay_config: RelayConfig) -> AsyncGenerator[RelayClient, None]:
    async with RelayClient(config=relay_config) as client:
        yield client


@pytest.fixture
async def mock_relay_client() -> AsyncGenerator[RelayClient, None]:
    client = RelayClient(config=RelayConfig(url="http://test"))
    yield client
    await client.disconnect()


## ==============================
## Broker Fixtures
## ==============================


@pytest.fixture(scope="session")
def broker_config() -> BrokerConfig:
    """Fixture that provides a configured BrokerConfig instance."""
    return BrokerConfig(url=BROKER_TEST_URL)


## ==============================
## Publisher Fixtures
## ==============================


@pytest.fixture
async def publisher_client(
    broker_config: BrokerConfig,
) -> AsyncGenerator[PublisherClient, None]:
    """Fixture that provides a configured PublisherClient instance."""

    async with PublisherClient(broker_config=broker_config) as client:
        yield client


## ==============================
## Worker Fixtures
## ==============================

DEFAULT_BROKER_PREFETCH_COUNT = 10


@pytest.fixture(scope="session")
def worker_config(
    relay_config: RelayConfig,
    broker_config: BrokerConfig,
    broker_prefetch_count: int = DEFAULT_BROKER_PREFETCH_COUNT,
) -> WorkerApplicationConfig:
    """Fixture that provides a configured WorkerConfig instance."""
    return WorkerApplicationConfig(
        kind=WORKER_KIND_NAME,
        name=WORKER_NAME,
        broker_config=broker_config,
        broker_prefetch_count=broker_prefetch_count,
    )
