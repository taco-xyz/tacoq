import asyncio
import json
import multiprocessing
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


# =============================
# Worker App Entrypoint
# =============================


def run_app(app_type: Literal["standalone", "task", "process"]):
    match app_type:
        # Run the worker application as its own standalone app. This is how you
        # would run it in a separate container by itself.
        case "standalone":
            asyncio.run(worker_app.entrypoint())

        # Run the worker as an asyncio task. Simple and allows you to run it
        # alongside the rest of your application.
        case "task":
            asyncio.create_task(worker_app.entrypoint())

        # Run the worker as a separate process. This is useful if you want to
        # run blocking tasks without blocking the main thread. Restrictions
        # imposed by Python's multiprocessing module may apply.
        case "process":

            def run_worker_app_process():
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
                loop.run_until_complete(worker_app.entrypoint())

            multiprocessing.Process(target=run_worker_app_process).start()


if __name__ == "__main__":
    run_app(app_type="standalone")
