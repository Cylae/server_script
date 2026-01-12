import sys
import os
from .core import system, config, security
from .services import portainer, media

def setup_wizard():
    print("\n--- Initial Setup ---")
    domain = config.get_auth_value("DOMAIN")
    if not domain:
        domain = input("Enter your domain name (e.g., example.com): ").strip()
        if domain:
            config.save_auth_value("DOMAIN", domain)

    email = config.get_auth_value("EMAIL")
    if not email:
        email = input("Enter your email for SSL (e.g., admin@example.com): ").strip()
        if email:
            config.save_auth_value("EMAIL", email)

    # Run security setup
    if input("Install security tools (UFW, Fail2Ban)? [Y/n]: ").lower() != 'n':
        security.install_security_tools()
        security.configure_firewall()
        security.configure_fail2ban()

def show_menu():
    print("\n--- Cylae Server Manager (Python Edition) ---")
    print("1. Install Core Dependencies (Docker)")
    print("2. Install Portainer")
    print("3. Install Plex")
    print("4. Uninstall Portainer")
    print("5. Uninstall Plex")
    print("6. Run Setup Wizard")
    print("0. Exit")
    return input("Select an option: ")

def main():
    system.check_root()

    # Check if setup is needed
    if not config.get_auth_value("DOMAIN"):
        print("First time run detected.")
        setup_wizard()

    # Initialize services
    srv_portainer = portainer.PortainerService()
    srv_plex = media.PlexService()

    while True:
        try:
            choice = show_menu()

            if choice == "1":
                from .core import docker_manager
                docker_manager.ensure_installed()
                print("Docker checked/installed.")
            elif choice == "2":
                srv_portainer.install()
            elif choice == "3":
                srv_plex.install()
            elif choice == "4":
                srv_portainer.uninstall()
            elif choice == "5":
                srv_plex.uninstall()
            elif choice == "6":
                setup_wizard()
            elif choice == "0":
                sys.exit(0)
            else:
                print("Invalid option.")
        except KeyboardInterrupt:
            print("\nExiting.")
            sys.exit(0)
        except Exception as e:
            print(f"Error: {e}")
            import traceback
            traceback.print_exc()
            input("Press Enter to continue...")

if __name__ == "__main__":
    main()
