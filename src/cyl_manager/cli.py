from typing import Optional
import typer
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

# Core & Services
from cyl_manager.core.logging import setup_logging, logger
from cyl_manager.core.system import SystemManager
from cyl_manager.core.orchestrator import InstallationOrchestrator
from cyl_manager.core.docker import DockerManager
from cyl_manager.ui.menu import Menu
from cyl_manager.services.registry import ServiceRegistry

# Import services to ensure they are registered
import cyl_manager.services.portainer  # pylint: disable=unused-import
import cyl_manager.services.gitea  # pylint: disable=unused-import
import cyl_manager.services.misc  # pylint: disable=unused-import
import cyl_manager.services.media  # pylint: disable=unused-import
import cyl_manager.services.more_services  # pylint: disable=unused-import
import cyl_manager.services.infrastructure  # pylint: disable=unused-import

app = typer.Typer(help="Cylae Server Manager CLI")
console = Console()

@app.command()
def install(service_name: str) -> None:
    """Install a specific service."""
    service_cls = ServiceRegistry.get(service_name)
    if not service_cls:
        logger.error("Service '%s' not found.", service_name)
        raise typer.Exit(code=1)

    svc = service_cls()
    if svc.is_installed:
        logger.info("%s is already installed.", svc.pretty_name)
    else:
        # CLI install is non-interactive by default for now
        # Call configure to set defaults/randoms if not set
        svc.configure()
        svc.install()

        summary = svc.get_install_summary()
        if summary:
            # pylint: disable=line-too-long
            console.print(Panel(summary, title=f"{svc.pretty_name} Installed", border_style="green"))

@app.command()
def install_all() -> None:
    """Install ALL services using the intelligent orchestrator."""
    services = []
    # Pre-configure
    for cls in ServiceRegistry.get_all().values():
        svc = cls()
        svc.configure()
        services.append(svc)

    logger.info("Initiating Full Stack Installation...")
    InstallationOrchestrator.install_services(services)

    console.print("\n[bold cyan]Installation Summary:[/bold cyan]")
    for svc in services:
        summary = svc.get_install_summary()
        if summary:
            console.print(Panel(summary, title=svc.pretty_name, border_style="green"))

@app.command()
def remove(service_name: str) -> None:
    """Remove a specific service."""
    service_cls = ServiceRegistry.get(service_name)
    if not service_cls:
        logger.error("Service '%s' not found.", service_name)
        raise typer.Exit(code=1)

    svc = service_cls()
    if not svc.is_installed:
        logger.warning("%s is not installed.", svc.pretty_name)
    else:
        svc.remove()

@app.command()
def status() -> None:
    """List status of all services."""
    table = Table(title="Service Status")
    table.add_column("Service Name", style="cyan", no_wrap=True)
    table.add_column("Status", style="magenta")
    table.add_column("URL", style="blue")

    # Optimization: Batch check all containers at once to avoid N+1 API calls
    # Fetching list of all containers is O(1) API call vs O(N) calls for checking each service.
    docker_manager = DockerManager()
    installed_containers = docker_manager.get_all_container_names()

    for name, cls in ServiceRegistry.get_all().items():
        # Optimization: Direct check against the pre-fetched set using Class Attribute
        # Avoids instantiating service classes for uninstalled services
        is_installed = cls.name in installed_containers

        status_text = "[green]Installed[/green]" if is_installed else "[red]Not Installed[/red]"

        url = ""
        if is_installed:
            svc = cls()
            url = svc.get_url() or ""

        table.add_row(name, status_text, url)

    console.print(table)

@app.command()
def menu() -> None:
    """Open the interactive menu."""
    Menu().run()

@app.callback()
def main(verbose: bool = False) -> None:
    """Cylae Server Manager."""
    level = "DEBUG" if verbose else "INFO"
    setup_logging(level)
    try:
        # Check privileges and OS compatibility
        try:
            SystemManager.check_root()
        except Exception as e:
            # Re-raising for production grade strictness.
            logger.error("%s", e)
            raise typer.Exit(1) from e

        SystemManager.check_os()

    except Exception as e:
        logger.error("Initialization error: %s", e)
        raise typer.Exit(1) from e

if __name__ == "__main__":
    app()
