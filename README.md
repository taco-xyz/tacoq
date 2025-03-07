# TacoQ

A multi-language distributed task queue

## Highlights

- 🚀 A **next-gen** alternative to `celery`.
- 🛠️ Works across multiple languages. E.g. **Route your tasks from a Python app to a Rust worker**.
- 🐍 **Idiomatic SDKs**: Native Python support for `asyncio`, `pydantic` and hot reloading.
- 🔩 Tier 2 support for languages without an SDK via **REST API**.
- 👀 **Built-in observability** with ready to use examples and an extendable interface.
- ⚡️ Backed by **RabbitMQ** (broker) and **Postgres** (storage). Super low latency.
- ⚙️ Distributed and **horizontally scalable**.

## Quick Start

### Server

Launch TacoQ using the following `docker-compose.yml`:

```yml
volumes:
  rabbitmq_data: {}
  postgres_data: {}

services:

  # ================================================
  # TacoQ Relay
  # The relay has two functions:
  # 1. Reads task updates from the message broker
  #    and writes them to the database.
  # 2. Has a REST API for getting tasks by ID.
  # ================================================

  relay:
    image: tacoq-relay:latest
    ports:
      - "3000:3000"
    depends_on:
      rabbitmq:
        condition: service_healthy
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 5s
      timeout: 5s
      retries: 5
    env_file:
      - .env.relay

  # ================================================
  # Broker
  # This is the message broker where all tasks get
  # routed through to the appropriate worker and to
  # the relay so it can save them to the database.
  # ================================================

  rabbitmq:
    image: rabbitmq:4-management
    ports:
      - "5672:5672"
      - "15672:15672"
    environment:
      RABBITMQ_DEFAULT_USER: user
      RABBITMQ_DEFAULT_PASS: password
    volumes:
      - rabbitmq_data:/var/lib/rabbitmq
    healthcheck:
      test: ["CMD", "rabbitmq-diagnostics", "check_port_connectivity"]
      interval: 5s
      timeout: 5s
      retries: 5

  # ================================================
  # Storage
  # This is the database where all tasks get saved.
  # ================================================

  postgres:
    image: postgres:latest
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
      POSTGRES_DB: tacoq
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d tacoq"]
      interval: 5s
      timeout: 5s
      retries: 5

```

### Client

Install it in your Python project using `pip` or `uv`:

```sh
pip install tacoq
```

```sh
uv add tacoq
```

### Client (Worker)

Set up your worker:

```py
# worker.py

import asyncio
import json
from typing import Any, Literal

# =============================
# Worker Setup
# =============================

from tacoq import (
    WorkerApplication,
    BrokerConfig,
    WorkerApplicationConfig,
    TaskInput,
    TaskOutput,
)

WORKER_KIND = "worker_waiter_kind"
TASK_KIND = "task_wait_n_seconds"

broker_config = BrokerConfig(url="amqp://user:password@localhost:5672")
worker_config = WorkerApplicationConfig(
    name="worker_waiter_1",
    kind=WORKER_KIND,
    broker_config=broker_config,
    broker_prefetch_count=5,
)

worker_app = WorkerApplication(config=worker_config)

# =============================
# Task Setup
# =============================


@worker_app.task(kind=TASK_KIND)
async def task_wait_n_seconds(input_data: TaskInput) -> TaskOutput:
    input_data_dict: dict[str, Any] = json.loads(input_data)
    seconds = input_data_dict.get("seconds", 0)
    await asyncio.sleep(seconds)

    return json.dumps(
        {
            "result": "Hello, world! You waited for %d seconds" % seconds,
            "seconds": seconds,
        }
    )


if __name__ == "__main__":
    asyncio.run(worker_app.entrypoint())
```

### Client (Publisher)

And you can now start publishing tasks:

```python
# publisher.py

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

async def main():
    # Serialize the input of the task to be a JSON string. All task inputs and outputs MUST be strings!
    task_input = json.dumps({"duration": duration})

    task = await publisher.publish_task(
        worker_kind=WORKER_KIND_NAME,
        task_kind=TASK_KIND_NAME,
        input_data=task_input,
    )

    # Fetch the task from the relay
    task = await publisher.get_task(task_id=task_id, retry_until_complete=True)

    # Print result
    if task.has_finished and task.output_data is not None:
        result = json.loads(task.output_data)
        print(result)
```

## Examples

View more in-depth examples in [`/examples`](https://github.com/taco-xyz/tacoq/tree/main/examples)