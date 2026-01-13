import typer
from typing import Optional
from .core.logging import setup_logging, logger
from .core.system import SystemManager
from .ui.menu import Menu
from .services.registry import ServiceRegistry
from .core.config import settings

# Import services to register them
import cyl_manager.services.portainer
import cyl_manager.services.gitea
import cyl_manager.services.misc
import cyl_manager.services.media
import cyl_manager.services.more_services
import cyl_manager.services.infrastructure

app = typer.Typer(help="Cylae Server Manager CLI")

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
    table = []
    for name, cls in ServiceRegistry.get_all().items():
        svc = cls()
        status = "Installed" if svc.is_installed else "Not Installed"
        print(f"{name:<20} : {status}")

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
        SystemManager.check_root()
    except Exception as e:
        logger.error(str(e))
        raise typer.Exit(1)

if __name__ == "__main__":
    app()
