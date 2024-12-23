import asyncio
import signal
from typing import Optional
from contextlib import AsyncExitStack
from worker.client import WorkerApplication


class ApplicationRunner:
    def __init__(self, app: WorkerApplication):
        self.app = app
        self._shutdown_event = asyncio.Event()
        self._task: Optional[asyncio.Task] = None

    async def startup(self, reload: bool = False):
        """Initialize and start the worker application."""

        def handle_shutdown(sig, _):
            self._shutdown_event.set()

        for sig in (signal.SIGTERM, signal.SIGINT):
            signal.signal(sig, handle_shutdown)

        self._task = asyncio.create_task(self.app.entrypoint())

        try:
            await self._shutdown_event.wait()
        except Exception as e:
            print(f"Error during application runtime: {e}")
        finally:
            await self.shutdown()

    async def shutdown(self):
        """Gracefully shutdown the application."""
        if self._task:
            try:
                self._task.cancel()
                await asyncio.wait_for(self._task, timeout=5.0)
            except asyncio.TimeoutError:
                print("Warning: Application shutdown timed out")
            except Exception as e:
                print(f"Error during shutdown: {e}")


async def run_application(
    app: WorkerApplication,
    reload: bool = False,
) -> None:
    """Run the worker application with lifecycle management."""
    async with AsyncExitStack():
        runner = ApplicationRunner(app)
        await runner.startup(reload)
