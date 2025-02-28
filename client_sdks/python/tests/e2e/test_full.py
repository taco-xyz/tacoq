from asyncio import sleep, create_task, gather
from typing import Coroutine, Any
import time
import uuid
import pytest
from datetime import datetime
from uuid import uuid4
import json

from models.task import Task, TaskInput, TaskOutput, TaskStatus
from publisher import PublisherClient
from worker import WorkerApplication
from worker.config import WorkerApplicationConfig
from broker.config import BrokerConfig
from logger_manager import LoggerManager, StructuredMessage as _
from tracer_manager import TracerManager
from manager.config import ManagerConfig

DELAYED_TASK = "delayed_task"
FAILING_TASK = "failing_task"
DELAYED_TASK_BLOCKING = "delayed_task_blocking"
QUICK_TASK = "quick_task"


async def delayed_task(input_data: TaskInput) -> TaskOutput:
    """Non-blocking task."""

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


async def quick_task(input_data: TaskInput) -> TaskOutput:
    """Quick task."""
    await sleep(0.1)
    return json.dumps({"message": "Task completed", "input": input_data})


async def delayed_task_blocking(input_data: TaskInput) -> TaskOutput:
    """Blocking task to test multiple processes."""
    time.sleep(2)

    return json.dumps({"message": "Task completed", "input": json.loads(input_data)})


async def failing_task(input_data: TaskInput) -> TaskOutput:
    """Failing task."""
    raise ValueError("Task failed intentionally")


class WorkerContext:
    """Context manager for running a worker in a separate process."""

    _worker_app: WorkerApplication = None  # type: ignore
    worker_kind: str = None  # type: ignore
    """ The kind of worker to use for this context. We generate it on the fly 
    so that we can run multiple workers in parallel and avoid queue collisions."""

    def __init__(self, broker_prefetch_count: int):
        self.worker_kind = str(uuid.uuid4())

        # Create and configure worker
        self._worker_app = WorkerApplication(
            config=WorkerApplicationConfig(
                name=self.worker_kind,
                kind=self.worker_kind,
                manager_config=ManagerConfig(url="http://localhost:3000"),
                broker_prefetch_count=broker_prefetch_count,
                broker_config=BrokerConfig(
                    url="amqp://user:password@localhost:5672",
                    test_mode=True,
                ),
            )
        )

        # Register appropriate task handler
        self._worker_app.register_task(DELAYED_TASK, delayed_task)
        self._worker_app.register_task(FAILING_TASK, failing_task)
        self._worker_app.register_task(DELAYED_TASK_BLOCKING, delayed_task_blocking)
        self._worker_app.register_task(QUICK_TASK, quick_task)

    async def __aenter__(self):
        # Run worker in background
        self._worker_task = create_task(self._worker_app.entrypoint())
        return self._worker_app

    async def __aexit__(self, exc_type, exc_val, exc_tb):  # type: ignore
        self._worker_app.issue_shutdown()
        await self._worker_app.wait_for_shutdown()


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.one
async def test_delayed_task_e2e():
    """Test a task that takes 2 seconds to complete.
    This test verifies the full lifecycle of a task:
    1. Task submission
    2. Immediate task status check (should be pending)
    3. Wait for completion
    4. Final task status check (should be completed)
    """
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    # Start worker in background
    async with WorkerContext(broker_prefetch_count=10) as worker:
        input_data = {"test": "data"}
        task = await publisher.publish_task(
            task_kind=DELAYED_TASK,
            worker_kind=worker.config.kind,
            input_data=json.dumps(input_data),
        )

        print(f"Published task {task}")
        await sleep(1)

        # Check immediate status
        task_status = await publisher.get_task(task.id)
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.PENDING
        assert task_status.output_data is None

        # Wait and check final status
        await sleep(3)  # Wait for task completion + buffer
        task_status = await publisher.get_task(task.id)
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.COMPLETED
        assert task_status.is_error == 0
        assert task_status.output_data is not None

        output_data = json.loads(task_status.output_data)

        assert output_data["message"] == "Task completed"
        assert json.loads(output_data["input"]) == input_data


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_parallel_delayed_tasks():
    """Test that multiple delayed tasks execute in parallel.
    Each task takes 2s, so 5 tasks should complete in ~2-3s if parallel,
    but would take 10s if sequential.
    """
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    async with WorkerContext(broker_prefetch_count=5) as worker:
        # Submit 5 delayed tasks
        tasks: list[Task] = []
        for i in range(5):
            task = await publisher.publish_task(
                task_kind=DELAYED_TASK,
                worker_kind=worker.config.kind,
                input_data=json.dumps({"task_num": i}),
            )
            tasks.append(task)

        # Wait for all tasks to complete
        await sleep(3)  # Should be enough time for parallel execution

        # Verify all tasks completed
        completed_tasks = 0
        for task in tasks:
            task_status = await publisher.get_task(task.id)
            assert task_status is not None
            if task_status.status == TaskStatus.COMPLETED:
                completed_tasks += 1
        assert completed_tasks == 5, (
            "Only %d / 5 tasks completed. Likely executed sequentially instead of in parallel"
            % completed_tasks
        )


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_error_task_e2e():
    """Test a task that fails immediately.
    This test verifies error handling:
    1. Task submission
    2. Task execution (fails)
    3. Task status check (should be failed)
    """
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    # Start worker in background
    async with WorkerContext(broker_prefetch_count=10) as worker:
        # Submit task
        task = await publisher.publish_task(
            task_kind=FAILING_TASK,
            worker_kind=worker.config.kind,
            input_data="",
        )

        # Wait a bit for task to be processed
        await sleep(1)

        # Check status
        task_status = await publisher.get_task(task.id)
        assert task_status is not None, "Task status is None"
        assert task_status.status == TaskStatus.COMPLETED
        assert task_status.is_error == 1
        assert task_status.output_data is not None
        assert task_status.output_data == "Task failed intentionally"


@pytest.mark.e2e
@pytest.mark.asyncio
async def test_task_not_found():
    """Test that requesting a non-existent task returns None"""
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    task_status = await publisher.get_task(uuid4())
    assert task_status is None


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.skip(reason="Multithreading is currently not supported")
async def test_parallel_blocking_tasks():
    """Test that tasks run in parallel with multiple workers"""
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    # Start worker with 2 processes
    async with WorkerContext(broker_prefetch_count=10) as worker:
        print("Started worker")
        # Submit two blocking tasks
        task1 = await publisher.publish_task(
            task_kind=DELAYED_TASK_BLOCKING,
            worker_kind=worker.config.kind,
            input_data="",
        )
        print("Published task 1")
        task2 = await publisher.publish_task(
            task_kind=DELAYED_TASK_BLOCKING,
            worker_kind=worker.config.kind,
            input_data="",
        )
        print("Published task 2")

        # Wait for both tasks to complete
        await sleep(2.5)

        # Check both completed
        task1_status = await publisher.get_task(task1.id)
        print(f"Task 1 status: {task1_status}")
        task2_status = await publisher.get_task(task2.id)
        print(f"Task 2 status: {task2_status}")

        assert task1_status is not None and task2_status is not None
        assert task1_status.status == TaskStatus.COMPLETED, (
            "Task 1 wasn't completed after 2.5s. Likely not running in parallel."
        )
        assert task2_status.status == TaskStatus.COMPLETED, (
            "Task 2 wasn't completed after 2.5s. Likely not running in parallel."
        )


@pytest.mark.e2e
@pytest.mark.asyncio
@pytest.mark.priority
async def test_priority_task():
    """Test that tasks run in parallel with multiple workers"""
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    async with WorkerContext(broker_prefetch_count=1) as worker:
        # Publish an initial task to for the rest of them to get stuck in
        print("Publishing initial task to enqueue the rest..")
        await publisher.publish_task(
            task_kind=DELAYED_TASK,
            worker_kind=worker.config.kind,
            input_data="",
            priority=1,
        )
        print("Published initial task!")
        coroutines: list[Coroutine[Any, Any, Task]] = []

        # We distribute a bunch of priorities at random and check if they're completed in the correct order
        TOTAL_TASKS = 13
        print(f"Publishing {TOTAL_TASKS} tasks at random priorities..")
        for priority in sorted(range(TOTAL_TASKS), key=lambda _: uuid4()):
            coroutines.append(
                publisher.publish_task(
                    task_kind=QUICK_TASK,
                    worker_kind=worker.config.kind,
                    input_data="",
                    priority=priority,
                )
            )
        print("Waiting for tasks to complete..")
        incomplete_tasks = await gather(*coroutines)
        # Wait for all tasks to complete and then gather the results so we can check when they were completed
        completed_tasks: list[Task] = []
        await sleep(TOTAL_TASKS * 0.1 + 3)
        print("Gathering results..")
        for task in incomplete_tasks:
            task = await publisher.get_task(task.id)
            assert task is not None, f"Task {task} was not found"
            completed_tasks.append(task)

        priority_completed_at: dict[int, datetime] = {}
        for task in completed_tasks:
            assert task.completed_at is not None, "Task was not completed"
            priority_completed_at[task.priority] = task.completed_at

        previous_completed_at = None
        previous_priority = -1
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
