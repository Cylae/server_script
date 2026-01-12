#!/bin/bash
set -euo pipefail

# ==============================================================================
# MAIN APPLICATION LOGIC
# ==============================================================================

# Source all libraries
source src/lib/core.sh
source src/lib/config.sh
source src/lib/security.sh
source src/lib/docker.sh
source src/lib/database.sh
source src/lib/proxy.sh
source src/lib/utils.sh
source src/lib/install_system.sh
source src/lib/media.sh

# Source Services
source src/services/gitea.sh
source src/services/nextcloud.sh
source src/services/vaultwarden.sh
source src/services/uptimekuma.sh
source src/services/wireguard.sh
source src/services/filebrowser.sh
source src/services/yourls.sh
source src/services/mail.sh
source src/services/netdata.sh
source src/services/portainer.sh
source src/services/ftp.sh
source src/services/glpi.sh

# Source Media Services
source src/services/plex.sh
source src/services/tautulli.sh
source src/services/jackett.sh
source src/services/sonarr.sh
source src/services/radarr.sh
source src/services/prowlarr.sh
source src/services/overseerr.sh
source src/services/qbittorrent.sh

show_dns_records() {
    local ip=$(curl -s https://api.ipify.org)
    msg "REQUIRED DNS RECORDS FOR $DOMAIN"
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "TYPE  | HOST                 | VALUE" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "A     | @                    | $ip" >&3
    echo -e "CNAME | www                  | $DOMAIN" >&3
    echo -e "CNAME | admin                | $DOMAIN" >&3

    if [ -d "/opt/gitea" ]; then echo -e "CNAME | git                  | $DOMAIN" >&3; fi
    if [ -d "/opt/nextcloud" ]; then echo -e "CNAME | cloud                | $DOMAIN" >&3; fi
    if [ -d "/opt/mail" ]; then echo -e "CNAME | mail                 | $DOMAIN" >&3; fi
    if [ -d "/opt/yourls" ]; then echo -e "CNAME | x                    | $DOMAIN" >&3; fi
    if [ -d "/opt/vaultwarden" ]; then echo -e "CNAME | pass                 | $DOMAIN" >&3; fi
    if [ -d "/opt/uptimekuma" ]; then echo -e "CNAME | status               | $DOMAIN" >&3; fi
    if [ -d "/opt/wireguard" ]; then echo -e "CNAME | vpn                  | $DOMAIN" >&3; fi
    if [ -d "/opt/filebrowser" ]; then echo -e "CNAME | files                | $DOMAIN" >&3; fi
    if docker ps --format '{{.Names}}' | grep -q "^portainer"; then echo -e "CNAME | portainer            | $DOMAIN" >&3; fi
    if docker ps --format '{{.Names}}' | grep -q "^netdata"; then echo -e "CNAME | netdata              | $DOMAIN" >&3; fi
    if [ -d "/opt/glpi" ]; then echo -e "CNAME | support              | $DOMAIN" >&3; fi

    # Media Stack DNS
    if [ -d "/opt/plex" ]; then echo -e "CNAME | plex                 | $DOMAIN" >&3; fi
    if [ -d "/opt/tautulli" ]; then echo -e "CNAME | tautulli             | $DOMAIN" >&3; fi
    if [ -d "/opt/jackett" ]; then echo -e "CNAME | jackett              | $DOMAIN" >&3; fi
    if [ -d "/opt/sonarr" ]; then echo -e "CNAME | sonarr               | $DOMAIN" >&3; fi
    if [ -d "/opt/radarr" ]; then echo -e "CNAME | radarr               | $DOMAIN" >&3; fi
    if [ -d "/opt/prowlarr" ]; then echo -e "CNAME | prowlarr             | $DOMAIN" >&3; fi
    if [ -d "/opt/overseerr" ]; then echo -e "CNAME | request              | $DOMAIN" >&3; fi
    if [ -d "/opt/qbittorrent" ]; then echo -e "CNAME | qbittorrent          | $DOMAIN" >&3; fi

    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "MX    | @                    | mail.$DOMAIN (Priority 10)" >&3
    echo -e "TXT   | @                    | v=spf1 mx ~all" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    ask "Press Enter to continue..." dummy
}

show_credentials() {
    clear >&3 || true
    msg "SAVED CREDENTIALS"
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3

    if [ -f "$AUTH_FILE" ]; then
        echo -e "${CYAN}--- Database ---${NC}" >&3
        grep "mysql_root_password" $AUTH_FILE | while read -r line; do
             echo "DB Root: ${line#*=}" >&3
        done || true
        grep "_db_pass" $AUTH_FILE | while read -r line; do
             local svc=$(echo "$line" | cut -d_ -f1)
             echo "$svc DB: ${line#*=}" >&3
        done || true

        echo -e "\n${CYAN}--- Services ---${NC}" >&3
        grep -v "mysql_root_password" $AUTH_FILE | grep -v "_db_pass" | while read -r line; do
             echo "${line%%=*}: ${line#*=}" >&3
        done || true
    else
        echo "No credentials found." >&3
    fi

    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "Credentials file: $AUTH_FILE" >&3
    ask "Press Enter to continue..." dummy
}

# Helper for installation status
is_installed() {
    local name=$1
    if docker ps --format '{{.Names}}' | grep -q "^$name"; then
        return 0
    fi
    if [ -d "/opt/$name" ] && [ -f "/opt/$name/docker-compose.yml" ]; then
        return 0
    fi
    return 1
}

status_str() {
    if is_installed "$1"; then echo -e "${GREEN}INSTALLED${NC}"; else echo -e "${RED}NOT INSTALLED${NC}"; fi
}

show_media_menu() {
    while true; do
        clear >&3 || true
        echo -e "${PURPLE}=================================================================${NC}" >&3
        echo -e "${PURPLE}  MEDIA STACK MANAGER${NC}" >&3
        echo -e "${PURPLE}=================================================================${NC}" >&3
        echo -e " 1. Manage Plex Media Server [$(status_str plex)]" >&3
        echo -e " 2. Manage Tautulli          [$(status_str tautulli)]" >&3
        echo -e " 3. Manage Sonarr (TV)       [$(status_str sonarr)]" >&3
        echo -e " 4. Manage Radarr (Movies)   [$(status_str radarr)]" >&3
        echo -e " 5. Manage Prowlarr (Index)  [$(status_str prowlarr)]" >&3
        echo -e " 6. Manage Jackett           [$(status_str jackett)]" >&3
        echo -e " 7. Manage Overseerr         [$(status_str overseerr)]" >&3
        echo -e " 8. Manage qBittorrent       [$(status_str qbittorrent)]" >&3
        echo -e "-----------------------------------------------------------------" >&3
        echo -e " 0. Return to Main Menu" >&3
        echo -e "-----------------------------------------------------------------" >&3

        ask "Select >" choice
        case $choice in
            1) manage_plex "$(is_installed "plex" && echo "remove" || echo "install")" ;;
            2) manage_tautulli "$(is_installed "tautulli" && echo "remove" || echo "install")" ;;
            3) manage_sonarr "$(is_installed "sonarr" && echo "remove" || echo "install")" ;;
            4) manage_radarr "$(is_installed "radarr" && echo "remove" || echo "install")" ;;
            5) manage_prowlarr "$(is_installed "prowlarr" && echo "remove" || echo "install")" ;;
            6) manage_jackett "$(is_installed "jackett" && echo "remove" || echo "install")" ;;
            7) manage_overseerr "$(is_installed "overseerr" && echo "remove" || echo "install")" ;;
            8) manage_qbittorrent "$(is_installed "qbittorrent" && echo "remove" || echo "install")" ;;
            0) return ;;
            *) echo "Invalid option" >&3; sleep 1 ;;
        esac

        # Auto-sync logic for media submenu
        if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ne 0 ]; then
             ask "Apply changes now (Update SSL/Nginx)? (y/n):" confirm
             if [[ "$confirm" == "y" ]]; then sync_infrastructure; fi
        fi
    done
}

show_menu() {
    clear >&3 || true
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "${PURPLE}  CYL.AE SERVER MANAGER (SECURE EDITION)${NC}" >&3
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "Domain: ${CYAN}$DOMAIN${NC} | IP: ${CYAN}$(ip -4 route get 1 | awk '{print $7}')${NC}" >&3
    echo -e "-----------------------------------------------------------------" >&3

    echo -e " 1. Manage Gitea           [$(status_str gitea)]" >&3
    echo -e " 2. Manage Nextcloud       [$(status_str nextcloud)]" >&3
    echo -e " 3. Manage Portainer       [$(status_str portainer)]" >&3
    echo -e " 4. Manage Netdata         [$(status_str netdata)]" >&3
    echo -e " 5. Manage Mail Server     [$(status_str mail)]" >&3
    echo -e " 6. Manage YOURLS          [$(status_str yourls)]" >&3
    echo -e " 7. Manage FTP             [$(if command -v vsftpd >/dev/null; then echo -e "${GREEN}INSTALLED${NC}"; else echo -e "${RED}NOT INSTALLED${NC}"; fi)]" >&3
    echo -e " 8. Manage Vaultwarden     [$(status_str vaultwarden)]" >&3
    echo -e " 9. Manage Uptime Kuma     [$(status_str uptimekuma)]" >&3
    echo -e "10. Manage WireGuard       [$(status_str wireguard)]" >&3
    echo -e "11. Manage FileBrowser     [$(status_str filebrowser)]" >&3
    echo -e "12. Manage GLPI (Ticket)   [$(status_str glpi)]" >&3
    echo -e "13. ${CYAN}Manage Media Stack -> ${NC}" >&3
    echo -e "-----------------------------------------------------------------" >&3
    echo -e " s. System Update" >&3
    echo -e " b. Backup Data" >&3
    echo -e " x. Restore from Backup" >&3
    echo -e " k. Health Check" >&3
    echo -e " r. Refresh Infrastructure (Nginx/SSL)" >&3
    echo -e " t. Tune System (Profile)" >&3
    echo -e " d. DNS Records Info" >&3
    echo -e " c. Show Credentials" >&3
    echo -e " h. Hardening & SSH" >&3
    echo -e " 0. Exit" >&3
    echo -e "-----------------------------------------------------------------" >&3
}

run_main() {
    # Initial Checks
    check_root
    check_os
    load_config
    init_system

    while true; do
        show_menu
        ask "Select >" choice

        case $choice in
            1) is_installed "gitea" && manage_gitea "remove" || manage_gitea "install" ;;
            2) is_installed "nextcloud" && manage_nextcloud "remove" || manage_nextcloud "install" ;;
            3) is_installed "portainer" && manage_portainer "remove" || manage_portainer "install" ;;
            4) is_installed "netdata" && manage_netdata "remove" || manage_netdata "install" ;;
            5) is_installed "mail" && manage_mail "remove" || manage_mail "install" ;;
            6) is_installed "yourls" && manage_yourls "remove" || manage_yourls "install" ;;
            7) command -v vsftpd &>/dev/null && manage_ftp "remove" || manage_ftp "install" ;;
            8) is_installed "vaultwarden" && manage_vaultwarden "remove" || manage_vaultwarden "install" ;;
            9) is_installed "uptimekuma" && manage_uptimekuma "remove" || manage_uptimekuma "install" ;;
            10) is_installed "wireguard" && manage_wireguard "remove" || manage_wireguard "install" ;;
            11) is_installed "filebrowser" && manage_filebrowser "remove" || manage_filebrowser "install" ;;
            12) is_installed "glpi" && manage_glpi "remove" || manage_glpi "install" ;;
            13) show_media_menu ;;

            s) system_update ;;
            b) manage_backup ;;
            x) manage_restore ;;
            k) health_check ;;
            r) sync_infrastructure ;;
            t) tune_system ;;
            d) show_dns_records ;;
            c) show_credentials ;;
            h) change_ssh_port ;;
            0) echo "Bye!" >&3; exit 0 ;;
            *) echo "Invalid option" >&3 ;;
        esac

        # Auto-sync logic for services
        if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -le 12 ] && [ "$choice" -ge 1 ]; then
            ask "Apply changes now (Update SSL/Nginx)? (y/n):" confirm
            if [[ "$confirm" == "y" ]]; then sync_infrastructure; fi
        else
            if [ "$choice" != "13" ] && [ "$choice" != "0" ]; then
                ask "Press Enter to continue..." dummy
            fi
        fi
    done
}
