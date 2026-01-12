#!/bin/bash
set -euo pipefail

# ==============================================================================
# MAIN APPLICATION LOGIC
# ==============================================================================

# Source all libraries
# Assuming we are running from the root directory or src/..
# But since this is main.sh, we expect it to be sourced by install.sh in root.
# So relative paths should be from root.

# If we are running this file directly, we need to handle paths.
# But the plan is to have a root install.sh that sources everything.

source src/lib/core.sh
source src/lib/config.sh
source src/lib/security.sh
source src/lib/docker.sh
source src/lib/database.sh
source src/lib/proxy.sh
source src/lib/utils.sh
source src/lib/media.sh
source src/lib/install_system.sh

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
# Media Stack
source src/services/qbittorrent.sh
source src/services/prowlarr.sh
source src/services/jackett.sh
source src/services/sonarr.sh
source src/services/radarr.sh
source src/services/plex.sh
source src/services/tautulli.sh
source src/services/overseerr.sh

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
    if [ -d "/opt/qbittorrent" ]; then echo -e "CNAME | qbittorrent          | $DOMAIN" >&3; fi
    if [ -d "/opt/prowlarr" ]; then echo -e "CNAME | prowlarr             | $DOMAIN" >&3; fi
    if [ -d "/opt/jackett" ]; then echo -e "CNAME | jackett              | $DOMAIN" >&3; fi
    if [ -d "/opt/sonarr" ]; then echo -e "CNAME | sonarr               | $DOMAIN" >&3; fi
    if [ -d "/opt/radarr" ]; then echo -e "CNAME | radarr               | $DOMAIN" >&3; fi
    if [ -d "/opt/plex" ]; then echo -e "CNAME | plex                 | $DOMAIN" >&3; fi
    if [ -d "/opt/tautulli" ]; then echo -e "CNAME | tautulli             | $DOMAIN" >&3; fi
    if [ -d "/opt/overseerr" ]; then echo -e "CNAME | overseerr            | $DOMAIN" >&3; fi

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

    # We read directly from AUTH_FILE
    if [ -f "$AUTH_FILE" ]; then
        # Mask passwords? No, the user wants to see them.
        # Maybe we should categorize them.

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

show_menu_media() {
    clear >&3 || true
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "${PURPLE}  MEDIA SUITE${NC}" >&3
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e " 1. Manage Plex Media Server [$(d_status plex)]" >&3
    echo -e " 2. Manage Tautulli          [$(d_status tautulli)]" >&3
    echo -e " 3. Manage Overseerr         [$(d_status overseerr)]" >&3
    echo -e " 4. Manage Sonarr (TV)       [$(d_status sonarr)]" >&3
    echo -e " 5. Manage Radarr (Movies)   [$(d_status radarr)]" >&3
    echo -e " 6. Manage Prowlarr (Index)  [$(d_status prowlarr)]" >&3
    echo -e " 7. Manage Jackett (Index)   [$(d_status jackett)]" >&3
    echo -e " 8. Manage qBittorrent       [$(d_status qbittorrent)]" >&3
    echo -e "-----------------------------------------------------------------" >&3
    echo -e " 0. Back to Main Menu" >&3
    echo -e "-----------------------------------------------------------------" >&3

    ask "Select >" mchoice
    case $mchoice in
        1) [ -d "/opt/plex" ] && manage_plex "remove" || manage_plex "install" ;;
        2) [ -d "/opt/tautulli" ] && manage_tautulli "remove" || manage_tautulli "install" ;;
        3) [ -d "/opt/overseerr" ] && manage_overseerr "remove" || manage_overseerr "install" ;;
        4) [ -d "/opt/sonarr" ] && manage_sonarr "remove" || manage_sonarr "install" ;;
        5) [ -d "/opt/radarr" ] && manage_radarr "remove" || manage_radarr "install" ;;
        6) [ -d "/opt/prowlarr" ] && manage_prowlarr "remove" || manage_prowlarr "install" ;;
        7) [ -d "/opt/jackett" ] && manage_jackett "remove" || manage_jackett "install" ;;
        8) [ -d "/opt/qbittorrent" ] && manage_qbittorrent "remove" || manage_qbittorrent "install" ;;
        0) return ;;
        *) echo "Invalid option" >&3; sleep 1 ;;
    esac

    # Auto-sync logic for media services
    if [[ "$mchoice" =~ ^[0-9]+$ ]] && [ "$mchoice" -le 8 ] && [ "$mchoice" -ge 1 ]; then
        ask "Apply changes now (Update SSL/Nginx)? (y/n):" confirm
        if [[ "$confirm" == "y" ]]; then sync_infrastructure; fi
    fi
}

show_menu() {
    clear >&3 || true
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "${PURPLE}  CYL.AE SERVER MANAGER (SECURE EDITION)${NC}" >&3
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "Domain: ${CYAN}$DOMAIN${NC} | IP: ${CYAN}$(ip -4 route get 1 | awk '{print $7}')${NC}" >&3
    echo -e "-----------------------------------------------------------------" >&3

    # Status Check Helper
    p_status() {
        if [ -d "/opt/$1" ]; then echo -e "${GREEN}INSTALLED${NC}"; else echo -e "${RED}NOT INSTALLED${NC}"; fi
    }
    d_status() {
        if docker ps --format '{{.Names}}' | grep -q "^$1"; then echo -e "${GREEN}INSTALLED${NC}"; else echo -e "${RED}NOT INSTALLED${NC}"; fi
    }

    echo -e " 1. Manage Gitea           [$(p_status gitea)]" >&3
    echo -e " 2. Manage Nextcloud       [$(p_status nextcloud)]" >&3
    echo -e " 3. Manage Portainer       [$(d_status portainer)]" >&3
    echo -e " 4. Manage Netdata         [$(d_status netdata)]" >&3
    echo -e " 5. Manage Mail Server     [$(p_status mail)]" >&3
    echo -e " 6. Manage YOURLS          [$(p_status yourls)]" >&3
    echo -e " 7. Manage FTP             [$(if command -v vsftpd >/dev/null; then echo -e "${GREEN}INSTALLED${NC}"; else echo -e "${RED}NOT INSTALLED${NC}"; fi)]" >&3
    echo -e " 8. Manage Vaultwarden     [$(p_status vaultwarden)]" >&3
    echo -e " 9. Manage Uptime Kuma     [$(p_status uptimekuma)]" >&3
    echo -e "10. Manage WireGuard       [$(p_status wireguard)]" >&3
    echo -e "11. Manage FileBrowser     [$(p_status filebrowser)]" >&3
    echo -e "12. Manage GLPI (Ticket)   [$(p_status glpi)]" >&3
    echo -e "13. Media Suite (Plex/Starr) >>" >&3
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
    setup_media_directories

    while true; do
        show_menu
        ask "Select >" choice

        case $choice in
            1) [ -d "/opt/gitea" ] && manage_gitea "remove" || manage_gitea "install" ;;
            2) [ -d "/opt/nextcloud" ] && manage_nextcloud "remove" || manage_nextcloud "install" ;;
            3) docker ps --format '{{.Names}}' | grep -q "^portainer" && manage_portainer "remove" || manage_portainer "install" ;;
            4) docker ps --format '{{.Names}}' | grep -q "^netdata" && manage_netdata "remove" || manage_netdata "install" ;;
            5) [ -d "/opt/mail" ] && manage_mail "remove" || manage_mail "install" ;;
            6) [ -d "/opt/yourls" ] && manage_yourls "remove" || manage_yourls "install" ;;
            7) command -v vsftpd &>/dev/null && manage_ftp "remove" || manage_ftp "install" ;;
            8) [ -d "/opt/vaultwarden" ] && manage_vaultwarden "remove" || manage_vaultwarden "install" ;;
            9) [ -d "/opt/uptimekuma" ] && manage_uptimekuma "remove" || manage_uptimekuma "install" ;;
            10) [ -d "/opt/wireguard" ] && manage_wireguard "remove" || manage_wireguard "install" ;;
            11) [ -d "/opt/filebrowser" ] && manage_filebrowser "remove" || manage_filebrowser "install" ;;
            12) [ -d "/opt/glpi" ] && manage_glpi "remove" || manage_glpi "install" ;;
            13) show_menu_media ;;

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
            if [ "$choice" != "0" ]; then
                ask "Press Enter to continue..." dummy
            fi
        fi
    done
}
