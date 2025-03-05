"""For these E2E tests to run, the following services must be running:

- Rust Manager
- RabbitMQ broker
- Postgres database

It is also recommended you run Grafana and Tempo to view the traces and logs.

To run the manager and broker, run the following command from the root directory:

```bash
docker compose up -d
```
"""

import asyncio
import json
import time
import uuid
from asyncio import create_task, gather, sleep
from datetime import datetime
from types import TracebackType
from typing import Any, Coroutine, Optional, Self, Type
from uuid import uuid4

import pytest
from opentelemetry.trace import get_current_span
from src.tacoq.core.infra.broker import BrokerConfig
from src.tacoq.core.infra.relay import RelayConfig
from src.tacoq.core.models import Task, TaskInput, TaskOutput, TaskStatus
from src.tacoq.core.telemetry import LoggerManager, TracerManager
from src.tacoq.core.telemetry import StructuredMessage as _
from src.tacoq.publisher import PublisherClient
from src.tacoq.worker import WorkerApplication, WorkerApplicationConfig

# =========================================
# Tasks
# =========================================

DELAYED_INSTRUMENTED_TASK = "delayed_instrumented_task"
"""Non-blocking task that emits spans and logs as it goes."""


async def delayed_instrumented_task(input_data: TaskInput) -> TaskOutput:
    await sleep(0.4)

    # We will emit a span an two logs so we can see in Grafana if these are being properly chained together
    logger = LoggerManager.get_logger()
    tracer = TracerManager.get_tracer()

    logger.info(
        _(
            message="Delayed task is at 20%",
            attributes={"percent": 20},
        )
    )

    with tracer.start_as_current_span("delayed_task_section"):
        await sleep(0.4)
        logger.info(
            _(
                message="Delayed task is at 40%",
                attributes={"percent": 40},
            )
        )
        await sleep(0.4)
        logger.info(
            _(
                message="Delayed task is at 60%",
                attributes={"percent": 60},
            )
        )
        await sleep(0.4)
        logger.info(
            _(
                message="Delayed task is at 80%",
                attributes={"percent": 80},
            )
        )

    await sleep(0.4)

    return json.dumps({"message": "Task completed", "input": input_data})


DELAYED_TASK_BLOCKING = "delayed_task_blocking"
"""Blocking task to test multiple processes."""


async def delayed_task_blocking(input_data: TaskInput) -> TaskOutput:
    time.sleep(2)

    return json.dumps({"message": "Task completed", "input": json.loads(input_data)})


VARIABLE_TASK = "variable_task"
""" Task whose duration varies with the input data. """


async def variable_task(input_data: TaskInput) -> TaskOutput:
    input: dict[str, Any] = json.loads(input_data)
    await sleep(input.get("delay", 0.1))

    return json.dumps({"message": "Task completed", "input": input})


FAILING_TASK = "failing_task"
"""Fails immediately."""


async def failing_task(input_data: TaskInput) -> TaskOutput:
    """Failing task."""
    raise ValueError("Task failed successfully")


# =========================================
# Worker Context
# This declares an async worker that runs
# in the same process asynchronously.
# =========================================


class WorkerContext:
    """Context manager for running a worker in a separate process."""

    _worker_app: WorkerApplication = None  # type: ignore
    worker_kind: str = None  # type: ignore
    """ The kind of worker to use for this context. We generate it on the fly 
    so that we can run multiple workers in parallel and avoid queue collisions."""

    def __init__(
        self: Self, broker_prefetch_count: int, worker_kind: Optional[str] = None
    ):
        if worker_kind is None:
            self.worker_kind = str(uuid.uuid4())
        else:
            self.worker_kind = worker_kind

        # Create and configure worker
        self._worker_app = WorkerApplication(
            config=WorkerApplicationConfig(
                name=self.worker_kind,
                kind=self.worker_kind,
                relay_config=RelayConfig(url="http://localhost:3000"),
                broker_prefetch_count=broker_prefetch_count,
                broker_config=BrokerConfig(
                    url="amqp://user:password@localhost:5672",
                    test_mode=True,
                ),
            )
        )

        # Register appropriate task handler
        self._worker_app.register_task(
            DELAYED_INSTRUMENTED_TASK, delayed_instrumented_task
        )
        self._worker_app.register_task(FAILING_TASK, failing_task)
        self._worker_app.register_task(DELAYED_TASK_BLOCKING, delayed_task_blocking)
        self._worker_app.register_task(VARIABLE_TASK, variable_task)

    async def __aenter__(self: Self) -> WorkerApplication:
        # Run worker in background
        self._worker_task = create_task(self._worker_app.entrypoint())
        return self._worker_app

    async def __aexit__(
        self: Self,
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> None:
        self._worker_app.issue_shutdown()
        await self._worker_app.wait_for_shutdown()


# =========================================
# Tests
# =========================================


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.timeout(15)
@pytest.mark.one
async def test_delayed_instrumented_task_e2e(publisher_client: PublisherClient):
    """Simple test: publishes one task and checks its status. We use an
    instrumented task so that we can see the spans and logs in Grafana.
    """
    current_span = get_current_span()

    async with WorkerContext(broker_prefetch_count=10) as worker:
        current_span.set_attribute("worker.kind", worker.config.kind)
        input_data = {"test": "data"}
        task = await publisher_client.publish_task(
            task_kind=DELAYED_INSTRUMENTED_TASK,
            worker_kind=worker.config.kind,
            input_data=json.dumps(input_data),
        )

        print(f"Published task {task}")

        # Wait and check final status
        task_status = await asyncio.wait_for(
            publisher_client.get_task(task.id, retry_until_complete=True),
            timeout=15,
        )
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.COMPLETED, (
            f"Task {task.id} is not completed"
        )
        assert task_status.is_error == 0
        assert task_status.output_data is not None

        output_data = json.loads(task_status.output_data)

        assert output_data["message"] == "Task completed"
        assert json.loads(output_data["input"]) == input_data


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_parallel_delayed_tasks(publisher_client: PublisherClient):
    """Tests multiple variable tasks executing in parallel and
    verifies that they were all executed within a shorter time than
    if they were executed sequentially."""

    TOTAL_TASKS = 50
    TIME_PER_TASK = 0.5
    TIME_EXPECTED = (TIME_PER_TASK * TOTAL_TASKS) / 2

    current_span = get_current_span()

    async with WorkerContext(broker_prefetch_count=TOTAL_TASKS + 1) as worker:
        current_span.set_attribute("worker.kind", worker.config.kind)
        coroutines: list[Coroutine[Any, Any, Task]] = []
        for i in range(TOTAL_TASKS):
            coroutines.append(
                publisher_client.publish_task(
                    task_kind=VARIABLE_TASK,
                    worker_kind=worker.config.kind,
                    input_data=json.dumps({"delay": TIME_PER_TASK, "task_num": i}),
                )
            )

        tasks = await gather(*coroutines)

        # Ensure all tasks are completed. This time should be shorter than TIME_PER_TASK * TOTAL_TASKS to verify that they were executed in parallel
        start_gathering = time.time()
        gather_tasks: list[Coroutine[Any, Any, Optional[Task]]] = []
        for task in tasks:
            gather_tasks.append(
                publisher_client.get_task(task.id, retry_until_complete=True)
            )
        await gather(*gather_tasks)
        time_taken = time.time() - start_gathering

        assert time_taken < TIME_EXPECTED, (
            f"Tasks were not executed in parallel. Time taken: {time_taken} seconds (expected {TIME_EXPECTED})"
        )


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.timeout(15)
async def test_error_task_e2e(publisher_client: PublisherClient):
    """Tests a task that fails immediately and checks that the
    serialized exception is properly returned."""

    current_span = get_current_span()

    async with WorkerContext(broker_prefetch_count=10) as worker:
        current_span.set_attribute("worker.kind", worker.config.kind)
        task = await publisher_client.publish_task(
            task_kind=FAILING_TASK,
            worker_kind=worker.config.kind,
            input_data="",
        )

        task_status = await publisher_client.get_task(
            task.id, retry_until_complete=True
        )
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.COMPLETED, (
            f"Task {task.id} is not completed"
        )
        assert task_status.is_error == 1, f"Task {task.id} is not an error"
        assert task_status.output_data is not None
        assert (
            "Task failed successfully" in task_status.output_data
            and "ValueError" in task_status.output_data
        )


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_task_not_found(publisher_client: PublisherClient):
    """Tests that requesting a non-existent task returns None."""
    task_status = await publisher_client.get_task(uuid4())
    assert task_status is None, "Task status is not None"


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.priority
async def test_priority_task(publisher_client: PublisherClient):
    """Tests that tasks are completed in the correct order when
    they have different priorities."""

    current_span = get_current_span()

    async with WorkerContext(broker_prefetch_count=1) as worker:
        current_span.set_attribute("worker.kind", worker.config.kind)
        # Publish an initial task to for the rest of them to get stuck in
        print("Publishing initial task to enqueue the rest..")
        await publisher_client.publish_task(
            task_kind=VARIABLE_TASK,
            worker_kind=worker.config.kind,
            input_data=json.dumps({"delay": 3}),
            priority=1,
        )
        print("Published initial task!")
        coroutines: list[Coroutine[Any, Any, Task]] = []

        # We distribute a bunch of priorities at random and check if they're completed in the correct order
        TOTAL_TASKS = 15
        print(f"Publishing {TOTAL_TASKS} tasks at random priorities..")
        for priority in sorted(range(TOTAL_TASKS), key=lambda _: uuid4()):
            coroutines.append(
                publisher_client.publish_task(
                    task_kind=VARIABLE_TASK,
                    worker_kind=worker.config.kind,
                    input_data=json.dumps({"delay": 0.1}),
                    priority=priority,
                )
            )
        print("Waiting for tasks to complete..")
        incomplete_tasks = await gather(*coroutines)
        # Wait for all tasks to complete and then gather the results so we can check when they were completed
        completed_tasks: list[Task] = []
        await sleep(TOTAL_TASKS * 0.1 + 5)
        print("Gathering results..")
        for task in incomplete_tasks:
            complete_task = await publisher_client.get_task(
                task.id, retry_until_complete=True
            )
            completed_tasks.append(complete_task)  # type: ignore

        priority_completed_at: dict[int, datetime] = {}
        for task in completed_tasks:
            assert task.completed_at is not None, "Task was not completed"
            priority_completed_at[task.priority] = task.completed_at

        previous_completed_at: Optional[datetime] = None
        previous_priority: Optional[int] = None
        ordered_priority_completed_at = sorted(
            priority_completed_at.items(), key=lambda x: x[1]
        )

        print("\nPriority order completion times:")
        for priority, completed_at in ordered_priority_completed_at:
            print(f"Priority {priority}: {completed_at}")

        for priority, completed_at in ordered_priority_completed_at:
            if previous_completed_at is not None:
                assert completed_at > previous_completed_at, (
                    f"Task of priority {priority} was completed before task of priority {previous_priority}"
                )
            previous_completed_at = completed_at
            previous_priority = priority


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.workers
async def test_multiple_workers_execute_tasks_in_parallel(
    publisher_client: PublisherClient,
):
    """Tests that tasks are executed in parallel when there are multiple workers."""

    TOTAL_WORKERS = 3
    BROKER_PREFETCH_COUNT = 5
    TOTAL_TASKS = TOTAL_WORKERS * BROKER_PREFETCH_COUNT
    TIME_PER_TASK = 1
    TIME_EXPECTED = TIME_PER_TASK * TOTAL_TASKS / TOTAL_WORKERS + 1
    worker_kind = str(uuid.uuid4())  # All workers must have the same kind

    current_span = get_current_span()

    current_span.set_attribute("worker.kind", worker_kind)
    worker_contexts: list[WorkerContext] = []
    for i in range(TOTAL_WORKERS):  # type: ignore
        worker_contexts.append(
            WorkerContext(
                broker_prefetch_count=BROKER_PREFETCH_COUNT, worker_kind=worker_kind
            )
        )

    for ctx in worker_contexts:
        await ctx.__aenter__()

    # Publish all tasks
    coroutines: list[Coroutine[Any, Any, Task]] = []
    for i in range(TOTAL_TASKS):  # type: ignore
        coroutines.append(
            publisher_client.publish_task(
                task_kind=VARIABLE_TASK,
                worker_kind=worker_contexts[0].worker_kind,
                input_data=json.dumps({"delay": TIME_PER_TASK}),
            )
        )

    incomplete_tasks = await gather(*coroutines)

    for ctx in worker_contexts:
        await ctx.__aexit__(None, None, None)  # type: ignore

    # Check that all tasks are completed
    start_gathering = time.time()
    gather_tasks: list[Coroutine[Any, Any, Optional[Task]]] = []
    for task in incomplete_tasks:
        gather_tasks.append(
            publisher_client.get_task(task.id, retry_until_complete=True)
        )
    await gather(*gather_tasks)
    time_taken = time.time() - start_gathering

    assert time_taken < TIME_EXPECTED, (
        f"Tasks were not executed in parallel. Time taken: {time_taken} seconds (expected {TIME_EXPECTED})"
    )
