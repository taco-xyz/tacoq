import asyncio
from uuid import UUID

from dataclasses import dataclass

from manager import ManagerClient, ManagerConfig
from models.task import TaskInput, TaskInstance


@dataclass
class PublisherClient:
    """A client for publishing tasks to the manager.

    ### Attributes
    - `manager_config`: The configuration for the manager.
    - `_manager_client`: The manager client.

    ### Methods
    - `publish_task`: Publish a task to the manager.
    - `get_task`: Get the status of a task by its UUID.
    """

    manager_config: ManagerConfig
    _manager_client: ManagerClient

    def __init__(self, manager_config: ManagerConfig):
        self._manager_config = manager_config
        self._manager_client = ManagerClient(manager_config)

    async def publish_task(self, task_kind: str, input_data: TaskInput) -> TaskInstance:
        """Publish a task to the manager.

        ### Arguments
        - `task_kind`: The kind of the task.
        - `input_data`: The data to publish.

        ### Returns
        - `TaskInstance`: The task instance.
        """

        return await self._manager_client.publish_task(task_kind, input_data)

    async def get_task(self, task_id: UUID, long_poll: bool = False) -> TaskInstance:
        """Get the status of a task by its UUID.

        ### Arguments
        - `task_id`: The UUID of the task.
        - `long_poll`: Whether to long poll for the task to finish.

        ### Returns
        - `TaskInstance`: The task instance.
        """

        task = await self._manager_client.get_task(task_id)

        if long_poll:
            while not task.has_finished:
                await asyncio.sleep(1)
                task = await self._manager_client.get_task(task_id)

        return task
