import asyncio
import signal
import traceback
from typing import Optional
from contextlib import AsyncExitStack
from worker.client import WorkerApplication
from cli.logger import logger
from cli.reloader import ModuleReloader
from cli.importer import import_from_string


class ApplicationRunner:
    """
    # Application Runner

    Handles the lifecycle of a TacoQ worker application including:
    - Signal handling
    - Graceful shutdown
    - Application startup and teardown
    """

    def __init__(self, app: WorkerApplication):
        """
        Initialize Application Runner

        ### Args:
            - `app`: WorkerApplication instance to manage
        """
        self.app = app
        self._shutdown_event = asyncio.Event()
        self._task: Optional[asyncio.Task] = None
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

    async def startup(self, reload: bool = False):
        """
        Initialize and start the worker application

        ### Args:
            `reload`: Enable hot reload mode
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
            done, pending = await asyncio.wait(
                [asyncio.create_task(self._shutdown_event.wait()), self._task],
                return_when=asyncio.FIRST_COMPLETED,
            )

            # If task completed first and has an exception, propagate it
            if self._task in done and self._task.exception():
                raise self._task.exception()

        except Exception:
            logger.error("Application crashed with traceback:")
            traceback.print_exc()
            raise
        finally:
            await self.shutdown()

    async def _run_with_reload(self):
        """Run the application with hot reload support"""
        reloader = ModuleReloader(self.app.__module__)
        app_task = None
        reload_task = None

        while True:
            if app_task is None:
                app_task = asyncio.create_task(self.app.entrypoint())

            if reload_task is None:
                reload_task = asyncio.create_task(reloader.watch_and_reload())
                reload_task = asyncio.shield(reload_task)

            try:
                done, pending = await asyncio.wait(
                    [reload_task, app_task], return_when=asyncio.FIRST_COMPLETED
                )

                if app_task in done:
                    # If app task completed or crashed, clean up and propagate the result
                    if reload_task and not reload_task.done():
                        reload_task.cancel()
                    return app_task.result()

                # At this point, the reload task must have completed
                if reload_task in done:
                    reload_occurred = reload_task.result()
                    reload_task = None  # Reset for next iteration

                    if reload_occurred:
                        logger.info("Detected changes, reloading application...")
                        # Cancel the current app task
                        if app_task and not app_task.done():
                            app_task.cancel()
                            try:
                                await app_task
                            except asyncio.CancelledError:
                                pass

                        # Re-import the application class
                        self.app = import_from_string(
                            f"{self.app.__module__}:{self.app.__class__.__name__}"
                        )
                        app_task = None  # Will be recreated in next iteration
                        continue

            except asyncio.CancelledError:
                # Handle external cancellation (e.g., SIGINT)
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

    ### Args:
        `app_import_string`: Import string for the WorkerApplication instance
        `reload`: Enable hot reload mode for development
    """
    async with AsyncExitStack():
        app = import_from_string(app_import_string)
        if not isinstance(app, WorkerApplication):
            raise TypeError("Application must be an instance of WorkerApplication")

        runner = ApplicationRunner(app)
        await runner.startup(reload)
