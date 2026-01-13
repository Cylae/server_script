import typer
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.prompt import Prompt
from ..services.registry import ServiceRegistry
from ..services.base import BaseService
from ..core.system import SystemManager
from ..core.config import settings, save_settings
from ..core.logging import print_header, logger

console = Console()

class Menu:
    def __init__(self):
        self.services = ServiceRegistry.get_all()

    def display_main_menu(self):
        print_header()

        # System Info
        profile = SystemManager.get_hardware_profile()
        table = Table(show_header=False, box=None)
        table.add_row(f"[bold cyan]Domain:[/bold cyan] {settings.DOMAIN}")
        table.add_row(f"[bold cyan]Profile:[/bold cyan] {profile}")
        console.print(Panel(table, title="System Info", expand=False))

        # Menu
        menu = Table(title="Main Menu", show_header=True, header_style="bold magenta")
        menu.add_column("Key", style="cyan", justify="center")
        menu.add_column("Action")
        menu.add_column("Status")

        i = 1
        service_map = {}
        for name, service_cls in self.services.items():
             # Quick check without instantiating fully if possible, but for now instantiate
            svc = service_cls()
            status = "[green]INSTALLED[/green]" if svc.is_installed else "[red]NOT INSTALLED[/red]"
            menu.add_row(str(i), f"Manage {svc.pretty_name}", status)
            service_map[str(i)] = svc
            i += 1

        console.print(menu)
        console.print("\n[bold]0.[/bold] Exit")

        return service_map

    def run(self):
        while True:
            console.clear()
            service_map = self.display_main_menu()
            choice = Prompt.ask("Select an option", default="0")

            if choice == "0":
                break

            if choice in service_map:
                self.manage_service(service_map[choice])
            else:
                console.print("[red]Invalid selection[/red]")
                Prompt.ask("Press Enter to continue")

    def manage_service(self, service: BaseService):
        console.clear()
        console.print(Panel(f"[bold]{service.pretty_name}[/bold]", style="cyan"))

        status = "Installed" if service.is_installed else "Not Installed"
        console.print(f"Status: [bold]{status}[/bold]")

        if service.is_installed:
            if Prompt.ask("Uninstall? (y/n)", default="n") == "y":
                service.remove()
        else:
            if Prompt.ask("Install? (y/n)", default="y") == "y":
                service.install()

        Prompt.ask("Press Enter to return to menu")
