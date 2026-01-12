import argparse
import sys
import shutil
import logging
from .core.logging import setup_logging
from .core.system import check_root, check_os, check_disk_space, install_core_dependencies
from .core.docker_runner import install_docker, ensure_network
from .core.proxy import sync_infrastructure
from .core.config import config
from .core.security import configure_firewall, install_fail2ban, harden_ssh
from .services.portainer import PortainerService
from .services.netdata import NetdataService

logger = logging.getLogger("cylae.cli")

def show_menu(services):
    print("\n" + "="*50)
    print(" CYLAE SERVER MANAGER (PYTHON EDITION)")
    print("="*50)

    for i, service in enumerate(services, 1):
        status = "INSTALLED" if service.status() else "NOT INSTALLED"
        print(f" {i}. Manage {service.name.capitalize()} [{status}]")

    print("-" * 50)
    print(" s. Sync Infrastructure (Nginx/SSL)")
    print(" h. Harden System (UFW, Fail2Ban, SSH)")
    print(" 0. Exit")
    print("-" * 50)

def main():
    parser = argparse.ArgumentParser(description="Cylae Server Manager (Python Edition)")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    # Setup logging
    setup_logging(debug=args.debug)

    # Root check (skip for help, but help is handled by argparse before this)
    try:
        check_root()
    except PermissionError as e:
        print(f"Error: {e}")
        print("Please run as root.")
        sys.exit(1)

    logger.info("Starting Cylae Server Manager...")

    # Core checks & Deps
    check_os()
    check_disk_space()
    try:
        install_core_dependencies()
    except Exception as e:
        logger.error(f"Failed to install core dependencies: {e}")
        # Continue? Maybe critical.

    # Ensure Config (Domain)
    try:
        config.ensure_domain()
    except KeyboardInterrupt:
        sys.exit(0)

    # Ensure Docker
    try:
        if not shutil.which("docker"):
             logger.info("Docker not found. Attempting install...")
             install_docker()
        ensure_network()
    except Exception as e:
        logger.error(f"Docker initialization failed: {e}")

    # Initialize Services
    services = [
        PortainerService(),
        NetdataService()
    ]

    while True:
        show_menu(services)
        try:
            choice = input("Select > ").strip()
        except KeyboardInterrupt:
            print("\nExiting...")
            break

        if choice == "0":
            print("Bye!")
            break

        if choice == "s":
            sync_infrastructure()
            input("Press Enter to continue...")
            continue

        if choice == "h":
            print("Applying security hardening...")
            configure_firewall()
            install_fail2ban()
            harden_ssh()
            print("Hardening complete.")
            input("Press Enter to continue...")
            continue

        try:
            idx = int(choice) - 1
            if 0 <= idx < len(services):
                service = services[idx]
                if service.status():
                    # Remove?
                    confirm = input(f"Remove {service.name}? (y/n): ").lower()
                    if confirm == 'y':
                        delete_data = input("Delete data? (y/n): ").lower() == 'y'
                        service.remove(delete_data)
                        # Ask to sync
                        if input("Sync Nginx/SSL now? (y/n): ").lower() == 'y':
                            sync_infrastructure()
                else:
                    # Install?
                    confirm = input(f"Install {service.name}? (y/n): ").lower()
                    if confirm == 'y':
                        service.install()
                        # Ask to sync
                        if input("Sync Nginx/SSL now? (y/n): ").lower() == 'y':
                            sync_infrastructure()
            else:
                print("Invalid selection.")
        except ValueError:
            print("Invalid input.")
        except Exception as e:
            logger.error(f"An error occurred: {e}")
            input("Press Enter to continue...")

if __name__ == "__main__":
    main()
