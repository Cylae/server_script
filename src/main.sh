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

show_menu() {
    clear >&3 || true
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "${PURPLE}  CYL.AE SERVER MANAGER (SECURE EDITION)${NC}" >&3
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "Domain: ${CYAN}$DOMAIN${NC} | IP: ${CYAN}$(ip -4 route get 1 | awk '{print $7}')${NC}" >&3
    echo -e "-----------------------------------------------------------------" >&3

    # Status Check Helper
    is_installed() {
        local name=$1
        # Check if docker container is running
        if docker ps --format '{{.Names}}' | grep -q "^$name"; then
            return 0
        fi
        # Fallback to directory check if service might be stopped but installed
        if [ -d "/opt/$name" ] && [ -f "/opt/$name/docker-compose.yml" ]; then
            return 0
        fi
        return 1
    }

    status_str() {
        if is_installed "$1"; then echo -e "${GREEN}INSTALLED${NC}"; else echo -e "${RED}NOT INSTALLED${NC}"; fi
    }

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
