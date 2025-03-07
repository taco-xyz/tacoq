# TacoQ Python SDK Examples

## 1. Simple Example
Launch a FastAPI app as the the task publisher and the worker as a separate container.

This example also includes explanations on running the worker in three different ways:
1. **As a single container** by using `asyncio.run(worker.entrypoint())`, making it so the worker is the only thing running.
2. **An asynchronous task** within the same application using `asyncio.create_task(worker.entrypoint())` as your FastAPI application. Only do this if your tasks are non-blocking as **a blocking task will block your entire app**;
3. **An entirely separate separate process** within the same Python application. This allows you to execute blocking tasks without worrying about stopping blocking your main application, but it introduces restrictions brought on by Python's multiprocessing module.