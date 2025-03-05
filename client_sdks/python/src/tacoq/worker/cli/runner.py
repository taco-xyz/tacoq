import asyncio
import signal
import traceback
from contextlib import AsyncExitStack
from typing import Any, Optional, Self

from tacoq.worker.cli.importer import import_from_string
from tacoq.worker.cli.logger import logger
from tacoq.worker.cli.reloader import ModuleReloader
from tacoq.worker.client import WorkerApplication


class ApplicationRunner:
    """
    # Application Runner

    Handles the lifecycle of a TacoQ worker application including:
    - Signal handling
    - Graceful shutdown
    - Application startup and teardown

    ### Attributes:
        - app: WorkerApplication instance to manage
        - import_string: Import string for the WorkerApplication instance

    ### Methods:
        - startup: Initialize and start the worker application
        - shutdown: Gracefully shutdown the application
    """

    def __init__(self: Self, app: WorkerApplication, import_string: str):
        """
        Initialize Application Runner

        ### Arguments:
            - app: WorkerApplication instance to manage
            - import_string: Import string for the WorkerApplication instance
        """
        self.app = app
        self._import_string = import_string
        self._shutdown_event = asyncio.Event()
        self._task: Optional[asyncio.Task[Any]] = None
        self._loop = asyncio.get_event_loop()

    def _handle_signals(self):
        """Configure signal handlers for graceful shutdown"""
        for sig in (signal.SIGTERM, signal.SIGINT):
            self._loop.add_signal_handler(
                sig, lambda: asyncio.create_task(self._signal_handler())
            )

    async def _signal_handler(self):
        """Handle shutdown signals"""
        logger.warning("Shutdown signal received...")
        self._shutdown_event.set()

    async def startup(self: Self, reload: bool = False):
        """
        Initialize and start the worker application

        ### Arguments:
            - reload: Enable hot reload mode
        """
        logger.info("Initializing application...")
        self._handle_signals()

        if reload:
            self._task = asyncio.create_task(self._run_with_reload())
        else:
            self._task = asyncio.create_task(self.app.entrypoint())
            logger.info("Application started successfully")

        try:
            # Wait for either shutdown signal or task completion
            done, _ = await asyncio.wait(
                [asyncio.create_task(self._shutdown_event.wait()), self._task],
                return_when=asyncio.FIRST_COMPLETED,
            )

            # If task completed first and has an exception, propagate it
            if self._task in done and self._task.exception():
                exception = self._task.exception()
                if exception is not None:
                    raise exception

        except Exception:
            logger.error("Application crashed with traceback:")
            traceback.print_exc()
            raise
        finally:
            await self.shutdown()

    async def _cleanup_app_task(self: Self, app_task: asyncio.Task[Any]) -> None:
        """Helper method to cleanup running application

        ### Arguments:
            - app_task: Task running the worker application
        """
        if not app_task.done():
            try:
                # Try graceful shutdown first
                if hasattr(self.app, "issue_shutdown"):
                    self.app.issue_shutdown()
                if hasattr(self.app, "wait_for_shutdown"):
                    await self.app.wait_for_shutdown()
            except asyncio.CancelledError:
                pass  # Suppress cleanup logs
            except asyncio.TimeoutError:
                logger.warning("Application cleanup timed out")
            except Exception as e:
                logger.error(f"Error during cleanup: {e}")

    async def _create_and_run_app_task(self) -> asyncio.Task[Any]:
        """Create and start the application task

        ### Returns:
            - `asyncio.Task`: Task running the worker application
        """
        return asyncio.create_task(self.app.entrypoint())

    async def _handle_task_result(self: Self, task: asyncio.Task[Any]) -> None:
        """Handle task completion and propagate exceptions

        ### Arguments:
            - task: Task to handle, can be any of the reload or worker tasks
        """
        if task.done() and task.exception():
            exception = task.exception()
            if exception is not None:
                raise exception

    async def _wait_for_completion(
        self: Self, *tasks: asyncio.Task[Any]
    ) -> set[asyncio.Task[Any]]:
        """Wait for any task to complete and return completed tasks

        ### Arguments:
            - tasks: Tasks to wait for
        """
        done, _ = await asyncio.wait(
            tasks,
            return_when=asyncio.FIRST_COMPLETED,
        )
        return done

    async def _run_with_reload(self):
        """Run the application with hot reload support"""
        reloader = ModuleReloader(self.app.__module__)
        app_task: Optional[asyncio.Task[Any]] = None
        reload_task: Optional[asyncio.Future[bool]] = None
        logger.info("Application started successfully")

        while True:
            self._shutdown_event.clear()

            if app_task is None:
                app_task = await self._create_and_run_app_task()

            if reload_task is None:
                reload_task = asyncio.shield(
                    asyncio.create_task(reloader.watch_and_reload())
                )
            try:
                done = await asyncio.gather(reload_task, app_task)

                if app_task in done:
                    if reload_task and not reload_task.done():
                        reload_task.cancel()
                    await self._handle_task_result(app_task)
                    return

                if reload_task in done and reload_task.result():
                    logger.info("Changes detected, restarting application...")
                    if app_task:
                        await self._cleanup_app_task(app_task)

                    self.app = import_from_string(self._import_string)
                    app_task = None
                    reload_task = None
                    logger.info("Application started successfully")
                    continue

            except asyncio.CancelledError:
                for task in [app_task, reload_task]:
                    if task and not task.done():
                        task.cancel()
                        try:
                            await task
                        except asyncio.CancelledError:
                            pass
                raise

    async def shutdown(self):
        """Gracefully shutdown the application"""
        if self._task and not self._task.done():
            try:
                logger.info("Initiating graceful shutdown...")
                self._task.cancel()
                await asyncio.wait_for(self._task, timeout=5.0)
                logger.info("Shutdown completed successfully")
            except asyncio.TimeoutError:
                logger.warning("Application shutdown timed out")
            except asyncio.CancelledError:
                logger.info("Application shutdown complete")
            except Exception as e:
                logger.error(f"Shutdown error: {e}")


async def run_application(
    app_import_string: str,
    reload: bool = False,
) -> None:
    """
    Run Worker Application

    Manages the complete lifecycle of a worker application.

    ### Arguments:
        - app_import_string: Import string for the WorkerApplication instance
        - reload: Enable hot reload mode for development
    """
    async with AsyncExitStack():
        app = import_from_string(app_import_string)
        if not isinstance(app, WorkerApplication):
            raise TypeError("Application must be an instance of WorkerApplication")

        runner = ApplicationRunner(app, app_import_string)
        await runner.startup(reload)
