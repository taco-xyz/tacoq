from typing import Optional

from uuid import UUID, uuid4
from aiohttp_retry import RetryOptionsBase

from broker import PublisherBrokerClient, BrokerConfig
from manager import ManagerClient, ManagerConfig
from models.task import TaskInput, Task, TaskKind


class PublisherClient:
    """A client for publishing and retrieving tasks."""

    # Broker
    _broker_config: BrokerConfig
    _broker_client: Optional[PublisherBrokerClient]

    # Manager
    _manager_config: ManagerConfig
    _manager_client: ManagerClient

    def __init__(self, manager_config: ManagerConfig, broker_config: BrokerConfig):
        self._manager_config = manager_config
        self._manager_client = ManagerClient(config=manager_config)

        self._broker_config = broker_config

    def _connect_to_broker(self):
        self._broker_client = PublisherBrokerClient(config=self._broker_config)

    async def publish_task(
        self,
        task_kind: str | TaskKind,
        input_data: Optional[TaskInput] = None,
        task_id: Optional[UUID] = None,
        priority: int = 0,
    ) -> Task:
        """Publish a task to the manager.

        ### Arguments
        - `task_kind`: The kind of the task. Can either be in the format of `worker_kind:task_name` string or a `TaskKind` object.
        - `input_data`: The data to publish.

        ### Returns
        - `TaskInstance`: The task instance.
        """

        # Connect to the broker if that hasn't yet been done
        if not self._broker_client:
            self._connect_to_broker()
        if not self._broker_client:
            raise Exception("Failed to connect to the broker")

        # Convert the task kind to a TaskKind if it is a string
        kind: TaskKind = (
            TaskKind.from_str(task_kind) if isinstance(task_kind, str) else task_kind
        )

        # Create a task with base values
        task = Task(
            id=task_id or uuid4(),
            task_kind=kind,
            input_data=input_data,
            priority=priority,
        )

        # Get the worker kind info
        worker_kind_info = await self._manager_client.get_worker_kind_broker_info(
            kind.worker_kind
        )

        # Publish the task to the manager
        await self._broker_client.publish_task(
            worker_kind_info.queue_name,
            task,
        )

        return task

    async def get_task(
        self, task_id: UUID, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> Task:
        """Get the status of a task by its UUID.

        ### Arguments
        - `task_id`: The UUID of the task.
        - `override_retry_options`: The retry options to use if you want to override the default ones.

        ### Returns
        - `Task`: The task.
        """

        return await self._manager_client.get_task(
            task_id, override_retry_options=override_retry_options
        )
