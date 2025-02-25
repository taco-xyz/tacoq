import asyncio
import json
from broker.config import BrokerConfig
from manager.config import ManagerConfig
from publisher.client import PublisherClient

# GENERAL CONFIGURATION _______________________________________________________
# These configs should be shared across both the publisher and the worker.

# Setup the manager location configuration
manager_config = ManagerConfig(url="http://localhost:3000")

# Setup the broker configuration
broker_config = BrokerConfig(
    url="amqp://user:password@localhost:5672", prefetch_count=5
)

# Both the publisher and the worker need to know about the task kinds and
# should have unified names for them.
WORKER_KIND_NAME = "worker_kind"
TASK_1_NAME = "task_1"
TASK_2_NAME = "task_2"

# APPLICATION CONFIGURATION ___________________________________________________

# 1. Create a producer application
worker_application = PublisherClient(
    manager_config=manager_config, broker_config=broker_config
)


# 2. Start the application
async def main():
    task1 = await worker_application.publish_task(
        TASK_1_NAME,
        WORKER_KIND_NAME,
        json.dumps({"data": "task_1_data"}),
    )
    task2 = await worker_application.publish_task(
        TASK_2_NAME,
        WORKER_KIND_NAME,
        json.dumps({"data": "task_2_data"}),
    )

    print(f"Task 1: {task1}")
    print(f"Task 2: {task2}")

    res1 = await worker_application.get_task(task1.id)
    res2 = await worker_application.get_task(task2.id)

    print(f"Task 1: {res1}")
    print(f"Task 2: {res2}")


if __name__ == "__main__":
    asyncio.run(main())
