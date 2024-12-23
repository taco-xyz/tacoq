import asyncio
from typing import Any

from broker import BrokerConfig
from manager.config import ManagerConfig
from worker import WorkerApplication, WorkerApplicationConfig

# GENERAL CONFIGURATION _______________________________________________________
# These configs should be shared across both the publisher and the worker.

# Both the publisher and the worker need to know about the manager.
manager_config = ManagerConfig(url="http://localhost:3000")
broker_config = BrokerConfig(
    url="amqp://user:password@localhost:5672", username="user", password="password"
)

# Both the publisher and the worker need to know about the task kinds and
# should have unified names for them.
TASK_1_NAME = "task_1"
TASK_2_NAME = "task_2"

# APPLICATION CONFIGURATION ___________________________________________________

# 2. Configure the worker
worker_config = WorkerApplicationConfig(
    name="test_worker",
    manager_config=manager_config,
    broker_config=broker_config,
)

# 3. Create a worker application
worker_application = WorkerApplication(worker_config)

# 4. Create tasks and register them with the worker application


@worker_application.task(TASK_1_NAME)
async def task_1(input_data: dict[Any, Any]) -> dict[Any, Any]:
    await asyncio.sleep(1)
    return input_data


@worker_application.task(TASK_2_NAME)
async def task_2(input_data: dict[Any, Any]) -> dict[Any, Any]:
    raise Exception("This is a test exception")


if __name__ == "__main__":
    asyncio.run(worker_application.entrypoint())
