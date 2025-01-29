import asyncio
from typing import Any, Callable
from uuid import uuid4
from pytest_benchmark.fixture import BenchmarkFixture  # type: ignore
import pytest

from src.manager import ManagerClient


def async_to_sync(func: Callable[..., Any]) -> Callable[..., Any]:
    """Decorator that wraps an async function to make it synchronous"""

    def wrapper(*args: Any, **kwargs: Any) -> Any:
        return asyncio.run(func(*args, **kwargs))

    return wrapper


## ====================================
## Async Functions
## ====================================


@async_to_sync
@pytest.mark.skip(reason="Not implemented")
async def create_n_tasks_sequential(
    manager_client: ManagerClient, test_task_kind: str, n: int
):
    """Creates n tasks with a given manager client"""

    for i in range(n):
        await manager_client.publish_task(test_task_kind, {"request_number": i})


@async_to_sync
@pytest.mark.skip(reason="Not implemented")
async def create_n_tasks_concurrent(
    manager_client: ManagerClient, test_task_kind: str, n: int
):
    """Creates n tasks with a given manager client"""

    await asyncio.gather(
        *[
            manager_client.publish_task(test_task_kind, {"request_number": i})
            for i in range(n)
        ]
    )


## ====================================
## Benchmark Suite
## ====================================


@pytest.fixture(params=[30])
def n_tasks(request: pytest.FixtureRequest) -> int:
    """Fixture that provides different numbers of tasks to create"""
    return request.param


@pytest.mark.skip(reason="Not implemented")
@pytest.mark.bench
def test_task_creation_benchmark_sync(
    benchmark: BenchmarkFixture, manager_client: ManagerClient, n_tasks: int
):
    """Benchmark synchronous task creation performance."""

    TEST_WORKER_NAME = str(uuid4())
    TEST_TASK_KIND = str(uuid4())

    # Register worker first
    worker_id = asyncio.run(
        manager_client.register_worker(TEST_WORKER_NAME, [TEST_TASK_KIND])
    )

    # Run benchmark
    try:
        benchmark.pedantic(  # type: ignore
            create_n_tasks_sequential,
            args=(manager_client, TEST_TASK_KIND, n_tasks),
            warmup_rounds=1,
            rounds=3,
            iterations=1,
        )

    # Unregister worker
    finally:
        asyncio.run(manager_client.unregister_worker(worker_id))


@pytest.mark.skip(reason="Not implemented")
@pytest.mark.bench
def test_task_creation_benchmark_concurrent(
    benchmark: BenchmarkFixture, manager_client: ManagerClient, n_tasks: int
):
    """Benchmark concurrent task creation performance."""

    TEST_WORKER_NAME = str(uuid4())
    TEST_TASK_KIND = str(uuid4())

    # Register worker first
    worker_id = asyncio.run(
        manager_client.register_worker(TEST_WORKER_NAME, [TEST_TASK_KIND])
    )

    # Run benchmark
    try:
        benchmark.pedantic(  # type: ignore
            create_n_tasks_concurrent,
            args=(manager_client, TEST_TASK_KIND, n_tasks),
            warmup_rounds=1,
            rounds=3,
            iterations=1,
        )

    # Unregister worker
    finally:
        asyncio.run(manager_client.unregister_worker(worker_id))
