"""Publisher client for the TacoQ client SDK.

The publisher client abstracts the details of communicating with
the broker and relay. This is a public-facing class that can
be used to publish tasks.
"""

from datetime import datetime
from typing import Any, Dict, Optional, TypeVar, Union
from uuid import UUID, uuid4

from opentelemetry.propagate import inject
from pydantic import BaseModel
from typing_extensions import Self

from tacoq.core.encoding import (
    Encoder,
    create_encoder,
)
from tacoq.core.infra.broker import BrokerConfig, PublisherBrokerClient
from tacoq.core.models import Task, TaskAssignmentUpdate
from tacoq.core.telemetry import TracerManager

InputType = TypeVar("InputType", bound=Union[bytes, str, Dict[str, Any], BaseModel])


class PublisherClient(BaseModel):
    """A client for publishing tasks to the message broker.

    NOTE: To retrieve tasks, refer to the `RelayClient` class.

    ### Attributes
    - broker_config: The configuration for the broker. See `BrokerConfig` for more details on how to configure it.

    ### Usage
    ```python
    from tacoq.core.infra.broker import BrokerConfig

    # Make sure RabbitMQ is running for tihs one!
    broker_config = BrokerConfig(
        host="localhost",
        port=5672,
    )
    publisher = PublisherClient(
        broker_config=broker_config
    )

    # Publish a task
    task = await publisher.publish_task(
        task_kind="task_name",
        worker_kind="worker_kind",
        input_data="Hello, world!",
        priority=5,
    )
    ```
    """

    # Broker
    broker_config: BrokerConfig
    """ The configuration for the broker. """

    _broker_client: Optional[PublisherBrokerClient] = None
    """ The broker client for publishing tasks. """

    async def publish_task(
        self: Self,
        task_kind: str,
        worker_kind: str,
        input_data: InputType,
        encoder: Optional[Encoder[InputType]] = None,
        task_id: Optional[UUID] = None,
        priority: int = 0,
        ttl_duration: int = 60 * 60 * 24 * 7,
        otel_ctx_carrier: Optional[Dict[str, str]] = None,
    ) -> Task:
        """Publish a task to the broker.

        ### Arguments:
        - task_kind: The kind of the task. Can either be in the format of
          `worker_kind:task_name` string or a `TaskKind` object.
        - worker_kind: The kind of worker that will execute the task.
        - input_data: The data to publish. The type of this data must be able
          to be encoded using the `encoder` function. By default, it accepts a
          Pydantic model.
        - encoder: The encoder function to use to encode the input data. If not
          provided, type hints will be used to infer the encoding logic.
        - task_id: The ID of the task. If not provided, a new UUID will be
          generated.
        - priority: The priority of the task.
        - ttl_duration: For how long the task should live after its done.
          Default value of 7 days.
        - otel_ctx_carrier: The OpenTelemetry context carrier to be added to
          the task. This will track the entire task's lifecycle. If none is
          provided, a new one will be created. If one is provided, the context
          is expected to already be injected.

        ### Returns
        - `Task`: The task instance.

        ### Behaviour
        - If you don't specify `encoder`, type hints are used to infer which
          encoder to use, fabricating one based on the types.
        - See more in `tacoq.core.encoding.polymorphic`.

        ### Usage
        ```python
        class TestInput(BaseModel):
            name: str

        # Using default encoder (inferred from type hints)
        task = await publisher.publish_task(
            task_kind="task_name",
            worker_kind="worker_kind",
            input_data=TestInput(name="input_data"),
            priority=5,
        )

        # Using custom encoder
        task = await publisher.publish_task(
            task_kind="task_name",
            worker_kind="worker_kind",
            input_data=TestInput(name="input_data"),
            encoder=CustomEncoder(),
            priority=5,
        )
        ```
        """

        # Get the tracer and begin a span
        tracer = TracerManager.get_tracer()
        with tracer.start_as_current_span("task_lifecycle") as span:
            # Connect to the broker if that hasn't yet been done
            if self._broker_client is None:
                with tracer.start_as_current_span("connect_to_broker"):
                    self._broker_client = PublisherBrokerClient(
                        config=self.broker_config
                    )

            # Inject context into the carrier
            if otel_ctx_carrier is None:
                otel_ctx_carrier = {}
            inject(otel_ctx_carrier)

            # Create encoder if not provided
            if encoder is None:
                encoder = create_encoder(type(input_data))

            # Create a task with base values
            encoded_input_data = encoder.encode(input_data)
            created_at = datetime.now()
            task = Task(
                id=task_id or uuid4(),
                task_kind=task_kind,
                worker_kind=worker_kind,
                input_data=encoded_input_data,
                priority=priority,
                ttl_duration=ttl_duration,
                otel_ctx_carrier=otel_ctx_carrier,
                created_at=created_at,
            )

            task_assignment_update = TaskAssignmentUpdate(
                id=task.id,
                task_kind=task_kind,
                worker_kind=worker_kind,
                input_data=encoded_input_data,
                priority=priority,
                ttl_duration=ttl_duration,
                otel_ctx_carrier=otel_ctx_carrier,
                created_at=created_at,
            )

            # Set the attributes of the span so it can be identified
            span.set_attributes(
                {
                    "task.id": str(task.id),
                    "task.kind": task_kind,
                    "worker.kind": worker_kind,
                }
            )

            # Publish the task to the broker
            with tracer.start_as_current_span("publish_task"):
                await self._broker_client.publish_task_assignment(
                    task_assignment_update
                )

            return task

    async def cleanup(self: Self) -> None:
        """Clean up the publisher client."""

        # Disconnect broker client
        if self._broker_client is not None:
            await self._broker_client.disconnect()

    # Context Management

    async def __aenter__(self: Self) -> Self:
        return self

    async def __aexit__(
        self: Self, exc_type: Any, exc_value: Any, traceback: Any
    ) -> None:
        await self.cleanup()
