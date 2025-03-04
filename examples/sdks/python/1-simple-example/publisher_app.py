import json
from typing import Any, Optional
from uuid import UUID
from pydantic import BaseModel

from fastapi import FastAPI

# =============================
# Publisher Setup
# =============================

from tacoq import PublisherClient, BrokerConfig, RelayConfig, Task

# These settings are based on the relay and broker configurations in the docker compose files.
broker_config = BrokerConfig(url="amqp://user:password@localhost:5672")
relay_config = RelayConfig(url="http://localhost:3000")
publisher = PublisherClient(broker_config=broker_config, relay_config=relay_config)

# These must be consistent with the worker app. Consider using a .env file to coordinate these.
WORKER_KIND_NAME = "worker_waiter_kind"
TASK_KIND_NAME = "task_wait_n_seconds"

# =============================
# Basic FastAPI Setup
# =============================

app = FastAPI()


class PublishTaskResponse(BaseModel):
    task_id: UUID
    """ The ID of the task that was published."""


@app.post("/task")
async def publish_task(duration: int):
    """
    Publishes a task to be worked on for `duration` seconds.
    """

    # Serialize the input of the task to be a JSON string. All task inputs and outputs MUST be strings!
    task_input = json.dumps({"duration": duration})

    task = await publisher.publish_task(
        worker_kind=WORKER_KIND_NAME,
        task_kind=TASK_KIND_NAME,
        input_data=task_input,
    )

    # Return the task ID to the client
    return PublishTaskResponse(task_id=task.id)


class FetchTaskResponse(BaseModel):
    task: Optional[
        Task
    ]  # The Task is a Pydantic BaseModel so it can be easily serialized.
    result: Optional[dict[str, Any]]


@app.get("/task/{task_id}")
async def fetch_task(task_id: UUID) -> FetchTaskResponse:
    """
    Fetches the task with the given ID.
    """

    # Fetch the task from the relay
    task = await publisher.get_task(task_id=task_id)

    if task is None:
        return FetchTaskResponse(task=None, result=None)

    # In this case, the result was serialized to a JSON string. You could use whatever serialization method
    # you want for the data, including Avro, Protocol Buffers, or even a custom binary format.
    if task.has_finished and task.output_data is not None:
        result = json.loads(task.output_data)
    else:
        result = None

    # Return the task to the client.
    return FetchTaskResponse(task=task, result=result)
