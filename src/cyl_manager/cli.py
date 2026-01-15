import typer
from typing import Optional
from rich.console import Console
from rich.table import Table

# Core & Services
from cyl_manager.core.logging import setup_logging, logger
from cyl_manager.core.system import SystemManager
from cyl_manager.core.orchestrator import InstallationOrchestrator
from cyl_manager.ui.menu import Menu
from cyl_manager.services.registry import ServiceRegistry

# Import services to ensure they are registered
import cyl_manager.services.portainer
import cyl_manager.services.gitea
import cyl_manager.services.misc
import cyl_manager.services.media
import cyl_manager.services.more_services
import cyl_manager.services.infrastructure

app = typer.Typer(help="Cylae Server Manager CLI")
console = Console()

@app.command()
def install(service_name: str):
    """Install a specific service."""
    service_cls = ServiceRegistry.get(service_name)
    if not service_cls:
        logger.error(f"Service '{service_name}' not found.")
        raise typer.Exit(code=1)

    svc = service_cls()
    if svc.is_installed:
        logger.info(f"{svc.pretty_name} is already installed.")
    else:
        svc.install()

@app.command()
def install_all():
    """Install ALL services using the intelligent orchestrator."""
    services = [cls() for cls in ServiceRegistry.get_all().values()]
    logger.info("Initiating Full Stack Installation...")
    InstallationOrchestrator.install_services(services)

@app.command()
def remove(service_name: str):
    """Remove a specific service."""
    service_cls = ServiceRegistry.get(service_name)
    if not service_cls:
        logger.error(f"Service '{service_name}' not found.")
        raise typer.Exit(code=1)

    svc = service_cls()
    if not svc.is_installed:
        logger.warning(f"{svc.pretty_name} is not installed.")
    else:
        svc.remove()

@app.command()
def status():
    """List status of all services."""
    table = Table(title="Service Status")
    table.add_column("Service Name", style="cyan", no_wrap=True)
    table.add_column("Status", style="magenta")

    for name, cls in ServiceRegistry.get_all().items():
        svc = cls()
        status_text = "[green]Installed[/green]" if svc.is_installed else "[red]Not Installed[/red]"
        table.add_row(name, status_text)

    console.print(table)

@app.command()
def menu():
    """Open the interactive menu."""
    Menu().run()

@app.callback()
def main(verbose: bool = False):
    """Cylae Server Manager."""
    level = "DEBUG" if verbose else "INFO"
    setup_logging(level)
    try:
        # Check privileges and OS compatibility
        try:
            SystemManager.check_root()
        except PermissionError as e:
            # Allow non-root for development/testing if needed, but warn heavily
            # or just exit as per requirement.
            # logger.error(str(e))
            # raise typer.Exit(1)
            # For now, we just warn if not root during development,
            # but in prod strict check is better.
            # Re-raising for production grade strictness.
            logger.error(str(e))
            raise typer.Exit(1)

        SystemManager.check_os()

    except Exception as e:
        logger.error(f"Initialization error: {e}")
        raise typer.Exit(1)

if __name__ == "__main__":
    app()
