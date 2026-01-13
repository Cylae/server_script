import sys
import concurrent.futures
from .core.system import check_root, check_os, determine_profile, check_disk_space
from .core.install_system import init_system_resources
from .core.docker import init_docker
from .core.security import harden_system
from .core.config import load_config, get, set_config
from .core.utils import ask, msg, header_art
from .services.plex import PlexService
from .services.portainer import PortainerService
from .services.gitea import GiteaService
from .services.misc import NextcloudService, VaultwardenService, UptimeKumaService, WireGuardService, FileBrowserService, YourlsService
from .services.more_services import MailService, NetdataService, FTPService, GLPIService
from .services.media_services import TautulliService, SonarrService, RadarrService, ProwlarrService, JackettService, OverseerrService, QbittorrentService
from .core.proxy import sync_infrastructure
from .core.backup import manage_backup, manage_restore

def status_str(service):
    return "INSTALLED" if service.is_installed() else "NOT INSTALLED"

def manage_service(service_class):
    svc = service_class()
    if svc.is_installed():
        if ask(f"Uninstall {svc.pretty_name}? (y/n)") == "y":
            svc.remove()
    else:
        svc.install()

def install_service_wrapper(service_class):
    """Wrapper for concurrent installation."""
    svc = service_class()
    if not svc.is_installed():
        msg(f"Installing {svc.pretty_name}...")
        svc.install()
        return f"{svc.pretty_name} Installed"
    return f"{svc.pretty_name} Already Installed"

def install_all_media_services():
    """Installs all media services with concurrency based on profile."""
    profile = determine_profile()
    services = [
        PlexService, TautulliService, SonarrService, RadarrService,
        ProwlarrService, JackettService, OverseerrService, QbittorrentService
    ]

    msg(f"Starting Bulk Installation. Hardware Profile: {profile}")

    # Check disk space before bulk install
    check_disk_space()

    if profile == "HIGH":
        msg("High Performance Profile: Installing in PARALLEL (Max 3 workers)")
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
            future_to_svc = {executor.submit(install_service_wrapper, svc): svc for svc in services}
            for future in concurrent.futures.as_completed(future_to_svc):
                try:
                    result = future.result()
                    msg(result)
                except Exception as e:
                    print(f"Service installation failed: {e}")
    else:
        msg("Low Spec Profile: Installing SEQUENTIALLY to prevent freeze")
        for svc_class in services:
            try:
                msg(install_service_wrapper(svc_class))
            except Exception as e:
                print(f"Service installation failed: {e}")

    if ask("Apply changes now (Update SSL/Nginx)? (y/n)") == "y":
        sync_infrastructure(get("EMAIL"))

def media_menu():
    while True:
        print("\n=== MEDIA STACK MANAGER ===")
        print(f"1. Manage Plex            [{status_str(PlexService())}]")
        print(f"2. Manage Tautulli        [{status_str(TautulliService())}]")
        print(f"3. Manage Sonarr (TV)     [{status_str(SonarrService())}]")
        print(f"4. Manage Radarr (Movies) [{status_str(RadarrService())}]")
        print(f"5. Manage Prowlarr (Index)[{status_str(ProwlarrService())}]")
        print(f"6. Manage Jackett         [{status_str(JackettService())}]")
        print(f"7. Manage Overseerr       [{status_str(OverseerrService())}]")
        print(f"8. Manage qBittorrent     [{status_str(QbittorrentService())}]")
        print("---------------------------------")
        print("99. Install ALL Media Services (Optimized)")
        print("0. Return to Main Menu")

        choice = ask("Select >")

        if choice == "1": manage_service(PlexService)
        elif choice == "2": manage_service(TautulliService)
        elif choice == "3": manage_service(SonarrService)
        elif choice == "4": manage_service(RadarrService)
        elif choice == "5": manage_service(ProwlarrService)
        elif choice == "6": manage_service(JackettService)
        elif choice == "7": manage_service(OverseerrService)
        elif choice == "8": manage_service(QbittorrentService)
        elif choice == "99": install_all_media_services()
        elif choice == "0": break

        if choice in [str(i) for i in range(1, 9)]:
            if ask("Apply changes now (Update SSL/Nginx)? (y/n)") == "y":
                sync_infrastructure(get("EMAIL"))

def main():
    check_root()
    check_os()
    load_config()
    init_system_resources()
    harden_system()

    # Initial hardware check on startup
    check_disk_space()

    # Ensure domain is set
    if get("DOMAIN") == "example.com":
        domain = ask("Enter your domain name (e.g., example.com)")
        set_config("DOMAIN", domain)
        email = ask("Enter your email for SSL alerts")
        set_config("EMAIL", email)

    while True:
        header_art()
        print(f"Domain: {get('DOMAIN')}")
        print("---------------------------------")
        print(f" 1. Manage Gitea           [{status_str(GiteaService())}]")
        print(f" 2. Manage Nextcloud       [{status_str(NextcloudService())}]")
        print(f" 3. Manage Portainer       [{status_str(PortainerService())}]")
        print(f" 4. Manage Netdata         [{status_str(NetdataService())}]")
        print(f" 5. Manage Mail Server     [{status_str(MailService())}]")
        print(f" 6. Manage YOURLS          [{status_str(YourlsService())}]")
        print(f" 7. Manage FTP             [{status_str(FTPService())}]")
        print(f" 8. Manage Vaultwarden     [{status_str(VaultwardenService())}]")
        print(f" 9. Manage Uptime Kuma     [{status_str(UptimeKumaService())}]")
        print(f"10. Manage WireGuard       [{status_str(WireGuardService())}]")
        print(f"11. Manage FileBrowser     [{status_str(FileBrowserService())}]")
        print(f"12. Manage GLPI (Ticket)   [{status_str(GLPIService())}]")
        print(f"13. Manage Media Stack ->")
        print("---------------------------------")
        print(" b. Backup Data")
        print(" x. Restore Data")
        print(" r. Refresh Infrastructure (Nginx/SSL)")
        print(" 0. Exit")

        choice = ask("Select >")

        if choice == "1": manage_service(GiteaService)
        elif choice == "2": manage_service(NextcloudService)
        elif choice == "3": manage_service(PortainerService)
        elif choice == "4": manage_service(NetdataService)
        elif choice == "5": manage_service(MailService)
        elif choice == "6": manage_service(YourlsService)
        elif choice == "7": manage_service(FTPService)
        elif choice == "8": manage_service(VaultwardenService)
        elif choice == "9": manage_service(UptimeKumaService)
        elif choice == "10": manage_service(WireGuardService)
        elif choice == "11": manage_service(FileBrowserService)
        elif choice == "12": manage_service(GLPIService)
        elif choice == "13": media_menu()
        elif choice == "b": manage_backup()
        elif choice == "x": manage_restore()
        elif choice == "r": sync_infrastructure(get("EMAIL"))
        elif choice == "0": break

        if choice in [str(i) for i in range(1, 13)]:
            if ask("Apply changes now (Update SSL/Nginx)? (y/n)") == "y":
                sync_infrastructure(get("EMAIL"))

if __name__ == "__main__":
    main()
