import click
import asyncio
from cli.runner import run_application

from cli.importer import import_from_string, ImportFromStringError


@click.group()
def cli():
    """CLI for managing the TacoQ Python workers."""
    pass


@cli.command()
@click.argument("app", type=str, required=True)
@click.option("--reload", is_flag=True, help="Enable live reload for development.")
def run(app: str, reload: bool):
    """
    # Run the Worker Application.

    ### Options:
    - --reload: Enable live reload in development.
    - --threads: Number of threads to enable multithreading.
    """
    try:
        worker_application = import_from_string(app)
    except ImportFromStringError as exc:
        click.echo(f"Error importing application: {exc}")
        return

    if reload:
        click.echo("Reload enabled (development mode).")
        # Live reload logic (e.g., file watching) can be implemented here.
    else:
        # Start the worker application with a single thread.
        asyncio.run(run_application(worker_application))
