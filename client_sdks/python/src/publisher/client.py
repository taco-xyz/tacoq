"""Publisher client for the TacoQ client SDK.

The publisher client abstracts the details of communicating with
the broker and manager. This is a public-facing class that can
be used to both publish tasks and retrieve their current status
and eventual output.
"""

from typing import Dict, Optional
from uuid import UUID, uuid4
import asyncio

from aiohttp_retry import RetryOptionsBase
from core.infra.broker import BrokerConfig, PublisherBrokerClient
from core.infra.manager import ManagerClient, ManagerConfig
from core.models import Task, TaskInput
from core.telemetry import TracerManager
from opentelemetry.propagate import inject
from pydantic import BaseModel
from typing_extensions import Self


class PublisherClient(BaseModel):
    """A client for publishing and retrieving tasks.

    ### Attributes
    - broker_config: The configuration for the broker. See `BrokerConfig` for more details on how to configure it.
    - manager_config: The configuration for the manager. See `ManagerConfig` for more details on how to configure it.

    ### Usage
    ```python
    from core.infra.broker import BrokerConfig
    from core.infra.manager import ManagerConfig

    # Make sure RabbitMQ is running for tihs one!
    broker_config = BrokerConfig(
        host="localhost",
        port=5672,
    )
    # Make sure the manager is running for this one!
    manager_config = ManagerConfig(
        host="localhost",
        port=8080,
    )
    publisher = PublisherClient(broker_config=broker_config, manager_config=manager_config)

    # Publish a task
    task = await publisher.publish_task(
        task_kind="task_name",
        worker_kind="worker_kind",
        input_data="Hello, world!",
        priority=5,
    )

    # Wait for the task to be processed...
    await asyncio.sleep(1)

    # Get the task
    task = await publisher.get_task(task.id)
    ```
    """

    # Broker
    broker_config: BrokerConfig
    """ The configuration for the broker. """

    _broker_client: Optional[PublisherBrokerClient] = None
    """ The broker client for publishing tasks. """

    # Manager
    manager_config: ManagerConfig
    """ The configuration for the manager. """

    _manager_client: ManagerClient = None  # type: ignore
    """ The manager client for retrieving tasks. """

    def model_post_init(self: Self, _) -> None:
        self._manager_client = ManagerClient(config=self.manager_config)

    def _connect_to_broker(self: Self) -> None:
        self._broker_client = PublisherBrokerClient(config=self.broker_config)

    async def publish_task(
        self: Self,
        task_kind: str,
        worker_kind: str,
        input_data: TaskInput = "",
        task_id: Optional[UUID] = None,
        priority: int = 0,
        otel_ctx_carrier: Optional[Dict[str, str]] = None,
    ) -> Task:
        """Publish a task to the manager.

        ### Arguments:
        - task_kind: The kind of the task. Can either be in the format of `worker_kind:task_name` string or a `TaskKind` object.
        - worker_kind: The kind of worker that will execute the task.
        - input_data: The data to publish.
        - task_id: The ID of the task. If not provided, a new UUID will be generated.
        - priority: The priority of the task.
        - otel_ctx_carrier: The OpenTelemetry context carrier to be added to the task. This will track the entire task's lifecycle. If none is provided, a new one will be created. If one is provided, the context is expected to already be injected.

        ### Returns
        - `TaskInstance`: The task instance.

        ### Usage
        You can publish a task with the following code:
        ```python
        task = await publisher.publish_task(
            task_kind="task_name",
            worker_kind="worker_kind",
            input_data="Hello, world!",
            priority=5,
        )
        ```
        You can also inject the OpenTelemetry context carrier to track the task's lifecycle:
        ```python
        otel_ctx_carrier = {"trace_id": "1234567890", "span_id": "1234567890"}
        task = await publisher.publish_task(
            task_kind="task_name",
            worker_kind="worker_kind",
            otel_ctx_carrier=otel_ctx_carrier,
        )
        ```
        """

        # Get the tracer and begin a span
        tracer = TracerManager.get_tracer()
        with tracer.start_as_current_span("task_lifecycle") as span:
            # Connect to the broker if that hasn't yet been done
            if self._broker_client is None:
                with tracer.start_as_current_span("connect_to_broker"):
                    self._connect_to_broker()
            if self._broker_client is None:
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
        self: Self,
        task_id: UUID,
        retry_until_complete: bool = False,
        override_retry_options: Optional[RetryOptionsBase] = None,
    ) -> Optional[Task]:
        """Get the status of a task by its UUID.

        ### Arguments:
        - task_id: The UUID of the task.
        - retry_until_complete: Whether to retry until the task is complete.
        - override_retry_options: The retry options to use if you want to override the default ones.

        ### Returns
        Task: The task.

        ### Usage
        You can get a task with the following code:
        ```python
        task = await publisher.get_task(task_id)
        ```
        The task ID must be saved at the time of publishing the task or it will
        become impossible to retrieve it.
        """

        task: Optional[Task] = None

        while task is None or not task.has_finished:
            task = await self._manager_client.get_task(
                task_id, override_retry_options=override_retry_options
            )
            if not retry_until_complete:
                break
            await asyncio.sleep(1)

        return task
