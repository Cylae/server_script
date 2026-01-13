import typer
from rich.console import Console
from rich.table import Table
from rich.prompt import Prompt, Confirm
from .core.registry import registry
from .core.config import config
from .core.docker_manager import docker_manager
from .core.system import check_root, get_hardware_specs
# Import services to register them
from .services import media_services # This will now work without circular dependency via __init__

app = typer.Typer()
console = Console()

def check_init():
    """Initial checks."""
    try:
        check_root()
        docker_manager.ensure_docker_installed()
        docker_manager.ensure_network()
    except Exception as e:
        console.print(f"[red]Initialization Error: {e}[/red]")
        raise typer.Exit(code=1)

@app.command()
def list():
    """List all available services and their status."""
    check_init()
    table = Table(title="Available Services")
    table.add_column("Service", style="cyan")
    table.add_column("Status", style="green")
    table.add_column("URL", style="blue")

    services = registry.get_all_services()
    for name, service in services.items():
        status = "[green]Installed[/green]" if service.is_installed() else "[yellow]Not Installed[/yellow]"
        # Port detection is basic here
        port = next(iter(service.ports.keys()), "N/A") if service.ports else "N/A"
        url = f"http://{config.DOMAIN}:{port}" if port != "N/A" else "N/A"
        table.add_row(service.pretty_name, status, url)

    console.print(table)

@app.command()
def install(service_name: str):
    """Install a specific service."""
    check_init()
    try:
        service = registry.get_service(service_name)
        service.install()
    except ValueError:
        console.print(f"[red]Service '{service_name}' not found.[/red]")

@app.command()
def remove(service_name: str, delete_data: bool = False):
    """Remove a specific service."""
    check_init()
    try:
        service = registry.get_service(service_name)
        if delete_data or Confirm.ask(f"Delete data for {service.pretty_name}?"):
            service.remove(delete_data=True)
        else:
            service.remove(delete_data=False)
    except ValueError:
        console.print(f"[red]Service '{service_name}' not found.[/red]")

@app.command()
def info():
    """Show system info."""
    specs = get_hardware_specs()
    console.print(specs)

@app.callback(invoke_without_command=True)
def main(ctx: typer.Context):
    """
    Cylae Server Manager
    """
    if ctx.invoked_subcommand is None:
        # Interactive Mode
        while True:
            console.clear()
            console.print("[bold blue]Cylae Server Manager[/bold blue]")

            # Re-implement list logic here to avoid clearing screen
            table = Table(title="Available Services")
            table.add_column("Service", style="cyan")
            table.add_column("Status", style="green")
            table.add_column("URL", style="blue")

            services = registry.get_all_services()
            for name, service in services.items():
                status = "[green]Installed[/green]" if service.is_installed() else "[yellow]Not Installed[/yellow]"
                port = next(iter(service.ports.keys()), "N/A") if service.ports else "N/A"
                url = f"http://{config.DOMAIN}:{port}" if port != "N/A" else "N/A"
                table.add_row(service.pretty_name, status, url)
            console.print(table)

            console.print("\n[bold]Actions:[/bold]")
            console.print("1. Install Service")
            console.print("2. Remove Service")
            console.print("0. Exit")

            choice = Prompt.ask("Select an action", choices=["1", "2", "0"])

            if choice == "0":
                raise typer.Exit()
            elif choice == "1":
                name = Prompt.ask("Enter service name (e.g., plex)")
                install(name)
                Prompt.ask("Press Enter to continue...")
            elif choice == "2":
                name = Prompt.ask("Enter service name (e.g., plex)")
                remove(name)
                Prompt.ask("Press Enter to continue...")

if __name__ == "__main__":
    app()
