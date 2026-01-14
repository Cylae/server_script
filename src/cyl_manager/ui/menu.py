import typer
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.prompt import Prompt
from ..services.registry import ServiceRegistry
from ..services.base import BaseService
from ..core.system import SystemManager
from ..core.orchestrator import InstallationOrchestrator
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
        table.add_row(f"[bold cyan]Email:[/bold cyan] {settings.EMAIL}")
        table.add_row(f"[bold cyan]Profile:[/bold cyan] {profile}")
        console.print(Panel(table, title="System Info", expand=False))

        # Menu
        menu = Table(title="Main Menu", show_header=True, header_style="bold magenta")
        menu.add_column("Key", style="cyan", justify="center")
        menu.add_column("Action")
        menu.add_column("Status")

        # Special Action
        menu.add_row("A", "[bold green]Full Stack Install[/bold green]", "Automated")

        i = 1
        service_map = {}
        for name, service_cls in self.services.items():
            svc = service_cls()
            status = "[green]INSTALLED[/green]" if svc.is_installed else "[red]NOT INSTALLED[/red]"
            menu.add_row(str(i), f"Manage {svc.pretty_name}", status)
            service_map[str(i)] = svc
            i += 1

        console.print(menu)
        console.print("\n[bold]c.[/bold] Configuration")
        console.print("[bold]0.[/bold] Exit")

        return service_map

    def sanitize_input(self, text: str) -> str:
        """Sanitizes input to remove surrogates and invalid characters."""
        if not text:
            return ""
        return text.encode("utf-8", "ignore").decode("utf-8").strip()

    def configure_settings(self):
        console.clear()
        console.print(Panel("[bold]Configuration[/bold]", style="cyan"))

        console.print(f"Current Domain: {settings.DOMAIN}")
        new_domain = Prompt.ask("Enter Domain Name", default=settings.DOMAIN)
        new_domain = self.sanitize_input(new_domain)

        console.print(f"Current Email: {settings.EMAIL}")
        new_email = Prompt.ask("Enter Admin Email", default=settings.EMAIL)
        new_email = self.sanitize_input(new_email)

        if new_domain != settings.DOMAIN or new_email != settings.EMAIL:
            save_settings("DOMAIN", new_domain)
            save_settings("EMAIL", new_email)
            settings.DOMAIN = new_domain
            settings.EMAIL = new_email
            console.print("[green]Settings saved successfully![/green]")
        else:
            console.print("[yellow]No changes made.[/yellow]")

        Prompt.ask("Press Enter to continue")

    def run_full_stack_install(self):
        console.clear()
        console.print(Panel("[bold green]Full Stack Installation[/bold green]", style="cyan"))
        console.print("This will install ALL services using the intelligent orchestrator.")

        if Prompt.ask("Are you sure?", choices=["y", "n"], default="y") == "y":
            services = [cls() for cls in self.services.values()]
            InstallationOrchestrator.install_services(services)
            console.print("[bold green]Full Stack Installation Process Completed.[/bold green]")

        Prompt.ask("Press Enter to return to menu")

    def run(self):
        # First Run Check
        if settings.DOMAIN == "example.com":
            console.print("[bold yellow]First Setup Detected![/bold yellow]")
            if Prompt.ask("Would you like to configure your domain and email now?", choices=["y", "n"], default="y") == "y":
                self.configure_settings()

        while True:
            console.clear()
            service_map = self.display_main_menu()
            choice = Prompt.ask("Select an option", default="0")

            if choice == "0":
                break
            elif choice == "c":
                self.configure_settings()
            elif choice.upper() == "A":
                self.run_full_stack_install()
            elif choice in service_map:
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
