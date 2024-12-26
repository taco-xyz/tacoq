import click
import asyncio
from cli.runner import run_application
from cli.importer import ImportFromStringError
from cli.logger import logger
from functools import wraps


def async_cmd(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        return asyncio.run(f(*args, **kwargs))

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
@async_cmd
async def run(app: str, reload: bool):
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
