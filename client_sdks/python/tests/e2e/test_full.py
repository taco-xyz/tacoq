from asyncio import sleep, get_event_loop
import time
import uuid
import pytest
from uuid import uuid4
import datetime
import json

from models.task import Task, TaskInput, TaskOutput, TaskStatus
from publisher import PublisherClient
from worker import WorkerApplication
from worker.config import WorkerApplicationConfig
from broker.config import BrokerConfig

from manager.config import ManagerConfig

DELAYED_TASK = "delayed_task"
FAILING_TASK = "failing_task"
DELAYED_TASK_BLOCKING = "delayed_task_blocking"


class WorkerContext:
    """Context manager for running a worker in a separate process."""

    _worker_app: WorkerApplication = None  # type: ignore
    worker_kind: str = None  # type: ignore
    """ The kind of worker to use for this context. We generate it on the fly 
    so that we can run multiple workers in parallel and avoid queue collisions."""

    def __init__(self, workers: int = 1):
        self.worker_kind = str(uuid.uuid4())

        async def delayed_task(input_data: TaskInput) -> TaskOutput:
            """Non-blocking task."""
            await sleep(2)

            return json.dumps({"message": "Task completed", "input": input_data})

        async def delayed_task_blocking(input_data: TaskInput) -> TaskOutput:
            """Blocking task to test multiple processes."""
            time.sleep(2)

            return json.dumps(
                {"message": "Task completed", "input": json.loads(input_data)}
            )

        async def failing_task(_: TaskInput) -> TaskOutput:
            raise ValueError("Task failed intentionally")

        # Create and configure worker
        self._worker_app = WorkerApplication(
            config=WorkerApplicationConfig(
                name=self.worker_kind,
                kind=self.worker_kind,
                workers=workers,
                manager_config=ManagerConfig(url="http://localhost:3000"),
                broker_config=BrokerConfig(
                    url="amqp://user:password@localhost:5672", test_mode=True
                ),
            )
        )

        # Register appropriate task handler
        self._worker_app.register_task(DELAYED_TASK, delayed_task)
        self._worker_app.register_task(FAILING_TASK, failing_task)
        self._worker_app.register_task(DELAYED_TASK_BLOCKING, delayed_task_blocking)

    def __enter__(self):
        # Run worker in background
        loop = get_event_loop()
        loop.create_task(self._worker_app.entrypoint())
        return self._worker_app

    def __exit__(self, exc_type, exc_val, exc_tb):  # type: ignore
        self._worker_app.shutdown()


@pytest.mark.e2e
@pytest.mark.asyncio
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
    with WorkerContext() as worker:
        input_data = {"test": "data"}
        task = await publisher.publish_task(
            task_kind=DELAYED_TASK,
            worker_kind=worker.config.kind,
            input_data=json.dumps(input_data),

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
        assert output_data["input"] == input_data


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

    with WorkerContext() as worker:
        start_time = datetime.datetime.now()

        # Submit 5 delayed tasks
        tasks: list[Task] = []
        for i in range(5):
            task = await publisher.publish_task(
                task_kind=DELAYED_TASK,
                worker_kind=worker.config.kind,
                input_data={"task_num": i},
            )
            tasks.append(task)

        # Wait for all tasks to complete
        await sleep(3)  # Should be enough time for parallel execution

        # Verify all tasks completed
        for task in tasks:
            task_status = await publisher.get_task(task.id)
            assert task_status is not None
            assert task_status.status == TaskStatus.COMPLETED
            assert task_status.is_error is False

        end_time = datetime.datetime.now()
        duration = (end_time - start_time).total_seconds()

        # Should take ~2-3s for parallel execution
        # Add some buffer for test environment variations
        assert duration < 4, (
            f"Tasks took {duration}s - likely executed sequentially instead of in parallel"
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
    with WorkerContext() as worker:
        # Submit task
        task = await publisher.publish_task(
            task_kind=FAILING_TASK,
            worker_kind=WORKER_KIND,
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
async def test_parallel_blocking_tasks():
    """Test that tasks run in parallel with multiple workers"""
    publisher = PublisherClient(
        manager_config=ManagerConfig(url="http://localhost:3000"),
        broker_config=BrokerConfig(url="amqp://user:password@localhost:5672"),
    )

    # Start worker with 2 processes
    with WorkerContext(workers=2) as worker:
        # Submit two blocking tasks
        task1 = await publisher.publish_task(
            task_kind=DELAYED_TASK_BLOCKING,
            worker_kind=worker.config.kind,
            input_data={},
        )
        task2 = await publisher.publish_task(
            task_kind=DELAYED_TASK_BLOCKING,
            worker_kind=worker.config.kind,
            input_data={},
        )

        # Wait for both tasks to complete
        await sleep(2.5)

        # Check both completed
        task1_status = await publisher.get_task(task1.id)
        task2_status = await publisher.get_task(task2.id)

        assert task1_status is not None and task2_status is not None
        assert task1_status.status == TaskStatus.COMPLETED, (
            "Task 1 wasn't completed after 2.5s. Likely not running in parallel."
        )
        assert task2_status.status == TaskStatus.COMPLETED, (
            "Task 2 wasn't completed after 2.5s. Likely not running in parallel."
        )
