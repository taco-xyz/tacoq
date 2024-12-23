import click
import asyncio
from cli.runner import run_application
from cli.importer import import_from_string, ImportFromStringError
from cli.logger import logger


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
def run(app: str, reload: bool):
    """
    Run Worker Application

    Launches a TacoQ worker application with the specified configuration.

    ### Arguments:
        `app`: The import path to the worker application (e.g., 'module.submodule:app')

    ### Options:
        --reload: Enable live reload for development mode
    """
    logger.info(f"Starting TacoQ worker application: {app}")

    try:
        logger.info("Importing worker application...")
        worker_application = import_from_string(app)
        logger.info("Application imported successfully")
    except ImportFromStringError as exc:
        logger.error(f"Failed to import application: {exc}")
        raise click.Abort()

    if reload:
        logger.warning("Development mode enabled (--reload)")
        logger.info("Live reload implementation pending")
    else:
        logger.info("Starting worker in production mode...")
        try:
            asyncio.run(run_application(worker_application))
        except Exception as e:
            logger.error(f"Application crashed: {e}")
            raise click.Abort()
