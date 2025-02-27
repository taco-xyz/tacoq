from typing import Optional, Dict
from uuid import UUID, uuid4
from aiohttp_retry import RetryOptionsBase
from opentelemetry.propagate import inject

from broker import PublisherBrokerClient, BrokerConfig
from manager import ManagerClient, ManagerConfig
from models.task import TaskInput, Task
from pydantic import BaseModel
from tracer_manager import TracerManager


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
        otel_ctx_carrier: Optional[Dict[str, str]] = None,
    ) -> Task:
        """Publish a task to the manager.

        ### Args:
        - task_kind: The kind of the task. Can either be in the format of `worker_kind:task_name` string or a `TaskKind` object.
        - worker_kind: The kind of worker that will execute the task.
        - input_data: The data to publish.
        - task_id: The ID of the task.
        - priority: The priority of the task.
        - otel_ctx_carrier: The OpenTelemetry context carrier to be added to the task. This will track the entire task's lifecycle. If none is provided, a new one will be created. If one is provided, the context is expected to already be injected.

        ### Returns
        - `TaskInstance`: The task instance.
        """

        # Get the tracer and begin a span
        tracer = TracerManager.get_tracer()
        with tracer.start_as_current_span("task_lifecycle") as span:
            # Connect to the broker if that hasn't yet been done
            if not self._broker_client:
                with tracer.start_as_current_span("connect_to_broker"):
                    self._connect_to_broker()
            if not self._broker_client:
                raise ConnectionError("Failed to connect to the broker")

            # Inject context into the carrier
            if otel_ctx_carrier is None:
                otel_ctx_carrier = {}
            inject(otel_ctx_carrier)

            # Create a task with base values
            task = Task(
                id=task_id or uuid4(),
                task_kind=task_kind,
                worker_kind=worker_kind,
                input_data=input_data,
                priority=priority,
                otel_ctx_carrier=otel_ctx_carrier,
            )

            # Set the attributes of the span so it can be identified
            span.set_attributes(
                {
                    "task.id": str(task.id),
                    "task.kind": task.task_kind,
                    "worker.kind": task.worker_kind,
                }
            )

            # Publish the task to the manager
            with tracer.start_as_current_span("publish_task"):
                await self._broker_client.publish_task(task)

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
