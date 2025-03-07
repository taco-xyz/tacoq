import asyncio
from functools import wraps
from typing import Any, Awaitable, Callable

import click
from tacoq.worker.cli.importer import ImportFromStringError
from tacoq.worker.cli.logger import logger
from tacoq.worker.cli.runner import run_application


def async_command(
    f: Callable[..., Awaitable[Any]],
) -> Callable[..., Any]:
    @wraps(f)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        return asyncio.run(f(*args, **kwargs))  # type: ignore

    return wrapper


@click.group()
def cli():
    """
    ### TacoQ CLI

    Command-line interface for managing TacoQ Python workers.
    Provides commands for running and managing worker applications.
    """
    pass


@cli.command()
@click.argument("app", type=str, required=True)
@click.option("--reload", is_flag=True, help="Enable live reload for development.")
@async_command
async def run(app: str, reload: bool) -> None:
    """Run Worker Application"""
    logger.info(f"Starting TacoQ worker application: {app}")

    if reload:
        logger.warning("Development mode enabled (--reload)")
    else:
        logger.info("Starting worker in production mode...")

    try:
        await run_application(app, reload=reload)
    except ImportFromStringError as exc:
        logger.error(f"Failed to import application: {exc}")
        raise click.Abort()
    except Exception as e:
        logger.error(f"Application crashed: {e}")
        raise click.Abort()
