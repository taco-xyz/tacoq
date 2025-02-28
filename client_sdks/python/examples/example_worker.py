import asyncio

from broker import BrokerConfig
from manager.config import ManagerConfig
from worker import WorkerApplication, WorkerApplicationConfig
from src.models import TaskInput, TaskOutput

# GENERAL CONFIGURATION _______________________________________________________
# These configs should be shared across both the publisher and the worker.

# The worker needs to know about the manager and the broker.
manager_config = ManagerConfig(url="http://localhost:3000")
broker_config = BrokerConfig(url="amqp://user:password@localhost:5672")

# Both the publisher and the worker need to know about the task kinds and
# should have unified names for them.
WORKER_KIND_NAME = "worker_kind"
TASK_1_NAME = "task_1"
TASK_2_NAME = "task_2"

# APPLICATION CONFIGURATION ___________________________________________________

# 1. Create a worker application
worker_application = WorkerApplication(
    config=WorkerApplicationConfig(
        name="test_worker",
        manager_config=manager_config,
        broker_config=broker_config,
        kind=WORKER_KIND_NAME,
        broker_prefetch_count=5,
    ),
)


# 2. Create tasks and register them with the worker application
@worker_application.task(TASK_1_NAME)
async def task_1(input_data: TaskInput) -> TaskOutput:
    await asyncio.sleep(1)
    return input_data


@worker_application.task(TASK_2_NAME)
async def task_2(_: TaskInput) -> TaskOutput:
    raise Exception("This is a test exception")


# 3. Run the worker application

if __name__ == "__main__":
    # Application can be run either as a standalone script or via the CLI.
    asyncio.run(worker_application.entrypoint())
