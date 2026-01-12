import sys
import os
from .core import system, config, security
from .services import (
    portainer, media, media_arr, downloaders,
    cloud, web, monitoring, network, database
)

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
    print("1. Install Core Dependencies")
    print("2. Setup Wizard")
    print("\n--- Media Stack ---")
    print("3. Install Plex")
    print("4. Install Sonarr")
    print("5. Install Radarr")
    print("6. Install Prowlarr")
    print("7. Install Jackett")
    print("8. Install Overseerr")
    print("9. Install Tautulli")
    print("10. Install qBittorrent")
    print("\n--- Cloud & Web ---")
    print("11. Install Nextcloud")
    print("12. Install FileBrowser")
    print("13. Install Vaultwarden")
    print("14. Install Gitea")
    print("15. Install GLPI")
    print("16. Install YourLS")
    print("\n--- System & Network ---")
    print("17. Install Portainer")
    print("18. Install Netdata")
    print("19. Install Uptime Kuma")
    print("20. Install WireGuard")
    print("21. Install FTP")
    print("22. Install Mail Server")
    print("23. Install MariaDB (Standalone)")
    print("\n0. Exit")
    return input("Select an option: ")

def main():
    system.check_root()

    # Check if setup is needed
    if not config.get_auth_value("DOMAIN"):
        print("First time run detected.")
        setup_wizard()
        # Auto install dependencies on first run
        from .core import docker_manager
        print("Checking core dependencies...")
        docker_manager.ensure_installed()

    # Service Registry
    services = {
        "3": media.PlexService(),
        "4": media_arr.SonarrService(),
        "5": media_arr.RadarrService(),
        "6": media_arr.ProwlarrService(),
        "7": media_arr.JackettService(),
        "8": media_arr.OverseerrService(),
        "9": media_arr.TautulliService(),
        "10": downloaders.QBittorrentService(),
        "11": cloud.NextcloudService(),
        "12": cloud.FileBrowserService(),
        "13": cloud.VaultwardenService(),
        "14": web.GiteaService(),
        "15": web.GLPIService(),
        "16": web.YourLSService(),
        "17": portainer.PortainerService(),
        "18": monitoring.NetdataService(),
        "19": monitoring.UptimeKumaService(),
        "20": network.WireGuardService(),
        "21": network.FTPService(),
        "22": network.MailService(),
        "23": database.MariaDBService()
    }

    from .core import docker_manager # Import here to use in loop

    while True:
        try:
            choice = show_menu()

            if choice == "0":
                sys.exit(0)
            elif choice == "1":
                docker_manager.ensure_installed()
                print("Docker checked/installed.")
            elif choice == "2":
                setup_wizard()
            elif choice in services:
                # Dependency check before install
                docker_manager.ensure_installed()

                srv = services[choice]
                action = input(f"1. Install {srv.display_name}\n2. Uninstall {srv.display_name}\nSelect: ")
                if action == "1":
                    srv.install()
                elif action == "2":
                    srv.uninstall()
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
