from typing import Optional
from uuid import UUID, uuid4
from aiohttp_retry import RetryOptionsBase

from broker import PublisherBrokerClient, BrokerConfig
from manager import ManagerClient, ManagerConfig
from models.task import TaskInput, Task
from pydantic import BaseModel


class PublisherClient(BaseModel):
    """A client for publishing and retrieving tasks."""

    # Broker
    broker_config: BrokerConfig
    _broker_client: Optional[PublisherBrokerClient] = None

    # Manager
    manager_config: ManagerConfig
    _manager_client: ManagerClient = None  # type: ignore

    def model_post_init(self, _) -> None:
        self._manager_client = ManagerClient(config=self.manager_config)

    def _connect_to_broker(self):
        self._broker_client = PublisherBrokerClient(config=self.broker_config)

    async def publish_task(
        self,
        task_kind: str,
        worker_kind: str,
        input_data: TaskInput = "",
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
            raise ConnectionError("Failed to connect to the broker")

        # Create a task with base values
        task = Task(
            id=task_id or uuid4(),
            task_kind=task_kind,
            worker_kind=worker_kind,
            input_data=input_data,
            priority=priority,
        )

        # Publish the task to the manager
        await self._broker_client.publish_task(
            task,
        )

        # Return the task
        return task

    async def get_task(
        self, task_id: UUID, override_retry_options: Optional[RetryOptionsBase] = None
    ) -> Optional[Task]:
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
