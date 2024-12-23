import click
import asyncio
from cli.runner import run_application
from cli.importer import import_from_string, ImportFromStringError


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
    click.echo(f"Starting TacoQ worker application: {app}")

    try:
        click.echo("Importing worker application...")
        worker_application = import_from_string(app)
        click.echo("Application imported successfully")
    except ImportFromStringError as exc:
        click.secho(f"ERROR: Failed to import application: {exc}", fg="red", err=True)
        raise click.Abort()

    if reload:
        click.secho("Development mode enabled (--reload)", fg="yellow", bold=True)
        # Live reload logic placeholder
        click.echo("Live reload implementation pending")
    else:
        click.echo("Starting worker in production mode...")
        try:
            asyncio.run(run_application(worker_application))
        except Exception as e:
            click.secho(f"ERROR: Application crashed: {e}", fg="red", err=True)
            raise click.Abort()
