import typer
from typing import Dict, Optional, List
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.prompt import Prompt

from cyl_manager.services.registry import ServiceRegistry
from cyl_manager.services.base import BaseService
from cyl_manager.core.system import SystemManager
from cyl_manager.core.orchestrator import InstallationOrchestrator
from cyl_manager.core.config import settings, save_settings
from cyl_manager.core.logging import print_header, logger

# Initialize Rich Console
console = Console()

class Menu:
    """
    Interactive CLI Menu for Cylae Server Manager.
    """
    def __init__(self) -> None:
        self.services = ServiceRegistry.get_all()

    def display_main_menu(self) -> Dict[str, BaseService]:
        """
        Displays the main menu and returns a map of keys to service instances.
        """
        print_header()

        # System Info Panel
        profile = SystemManager.get_hardware_profile()
        info_table = Table(show_header=False, box=None)
        info_table.add_row(f"[bold cyan]Domain:[/bold cyan] {settings.DOMAIN}")
        info_table.add_row(f"[bold cyan]Email:[/bold cyan] {settings.EMAIL}")
        info_table.add_row(f"[bold cyan]Profile:[/bold cyan] {profile}")
        console.print(Panel(info_table, title="System Info", expand=False))

        # Main Menu Table
        menu_table = Table(title="Main Menu", show_header=True, header_style="bold magenta")
        menu_table.add_column("Key", style="cyan", justify="center")
        menu_table.add_column("Action")
        menu_table.add_column("Status")

        # Global Actions
        menu_table.add_row("A", "[bold green]Full Stack Install[/bold green]", "Automated")

        # Service List
        i = 1
        service_map: Dict[str, BaseService] = {}
        for _, service_cls in self.services.items():
            # Instantiate lightly to check status
            # Ideally, is_installed shouldn't be too heavy.
            svc = service_cls()
            status = "[green]INSTALLED[/green]" if svc.is_installed else "[red]NOT INSTALLED[/red]"
            menu_table.add_row(str(i), f"Manage {svc.pretty_name}", status)
            service_map[str(i)] = svc
            i += 1

        console.print(menu_table)
        console.print("\n[bold]c.[/bold] Configuration")
        console.print("[bold]0.[/bold] Exit")

        return service_map

    def sanitize_input(self, text: str) -> str:
        """Sanitizes input to remove surrogates and invalid characters."""
        if not text:
            return ""
        try:
            return text.encode("utf-8", "ignore").decode("utf-8").strip()
        except UnicodeError:
            return ""

    def configure_settings(self) -> None:
        """
        Interactive configuration wizard.
        """
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
            # No need to manually update settings object here as save_settings reloads it
            console.print("[green]Settings saved successfully![/green]")
        else:
            console.print("[yellow]No changes made.[/yellow]")

        Prompt.ask("Press Enter to continue")

    def run_full_stack_install(self) -> None:
        """
        Triggers the installation of all services.
        """
        console.clear()
        console.print(Panel("[bold green]Full Stack Installation[/bold green]", style="cyan"))
        console.print("This will install ALL services using the intelligent orchestrator.")

        if Prompt.ask("Are you sure?", choices=["y", "n"], default="y") == "y":
            services = [cls() for cls in self.services.values()]
            InstallationOrchestrator.install_services(services)
            console.print("[bold green]Full Stack Installation Process Completed.[/bold green]")

        Prompt.ask("Press Enter to return to menu")

    def manage_service(self, service: BaseService) -> None:
        """
        Sub-menu for managing a specific service.
        """
        console.clear()
        console.print(Panel(f"[bold]{service.pretty_name}[/bold]", style="cyan"))

        status = "Installed" if service.is_installed else "Not Installed"
        console.print(f"Status: [bold]{status}[/bold]")

        if service.is_installed:
            if Prompt.ask("Uninstall?", choices=["y", "n"], default="n") == "y":
                service.remove()
        else:
            if Prompt.ask("Install?", choices=["y", "n"], default="y") == "y":
                service.install()

        Prompt.ask("Press Enter to return to menu")

    def run(self) -> None:
        """
        Main loop for the interactive menu.
        """
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
                console.print("[bold cyan]Goodbye![/bold cyan]")
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
