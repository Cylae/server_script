#!/bin/bash
# ==============================================================================
#  CYL.AE SERVER MANAGER V8.0 (Refactored & Secured)
#  The best script EVER for managing your self-hosted services.
# ==============================================================================

# Strict mode: exit on error, undefined vars, or pipe failures.
set -u

# ------------------------------------------------------------------------------
# 1. CONFIGURATION & CONSTANTS
# ------------------------------------------------------------------------------

CONFIG_FILE="/etc/cyl_manager.conf"
LOG_FILE="/var/log/server_manager.log"
AUTH_FILE="/root/.auth_details"
DOCKER_NET="server-net"
BACKUP_DIR="/var/backups/cyl_manager"

# Redirect all output to log file, but keep fd 3 for user interaction
exec 3>&1 1>>"$LOG_FILE" 2>&1

# ------------------------------------------------------------------------------
# 2. LOAD MODULES
# ------------------------------------------------------------------------------

# Get directory of this script
SCRIPT_DIR="$(dirname "$(realpath "$0")")"

# Source libraries
source "$SCRIPT_DIR/lib/utils.sh"
source "$SCRIPT_DIR/lib/core.sh"
source "$SCRIPT_DIR/lib/services.sh"

# ------------------------------------------------------------------------------
# 3. MENU & HOUSEKEEPING
# ------------------------------------------------------------------------------

manage_backup() {
    msg "Starting Backup..."
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    
    # DB Dump
    mysqldump -u root --password="$DB_ROOT_PASS" --all-databases > "$BACKUP_DIR/db_$TIMESTAMP.sql"
    
    # Files
    tar -czf "$BACKUP_DIR/files_$TIMESTAMP.tar.gz" /opt /var/www /etc/nginx/sites-available /root/.auth_details 2>/dev/null
    
    # Retention (keep last 5)
    ls -tp "$BACKUP_DIR"/db_*.sql | grep -v '/$' | tail -n +6 | xargs -I {} rm -- {} 2>/dev/null || true
    ls -tp "$BACKUP_DIR"/files_*.tar.gz | grep -v '/$' | tail -n +6 | xargs -I {} rm -- {} 2>/dev/null || true
    
    success "Backup Complete: $TIMESTAMP"
}

system_update() {
    msg "Updating System..."
    apt-get update && apt-get upgrade -y >/dev/null
    
    msg "Updating Docker Images..."
    if docker ps >/dev/null 2>&1; then
        docker images --format "{{.Repository}}:{{.Tag}}" | grep -v "<none>" | xargs -L1 docker pull 2>/dev/null || true
        docker image prune -f >/dev/null
    fi
    success "System Updated"
}

manage_ssh() {
    msg "Hardening SSH Security..."
    
    if [ ! -f /root/.ssh/authorized_keys ] || [ ! -s /root/.ssh/authorized_keys ]; then
        warn "NO SSH KEYS FOUND!"
        echo "You must add your public key to /root/.ssh/authorized_keys before disabling passwords."
        ask "Abort? (y/n):" confirm
        if [[ "$confirm" != "n" ]]; then return; fi
    fi
    
    cp /etc/ssh/sshd_config /etc/ssh/sshd_config.bak
    
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
    sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
    sed -i 's/PermitRootLogin yes/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config

    ask "Change SSH Port? (current: 22) [y/N]:" change_port
    if [[ "$change_port" =~ ^[Yy]$ ]]; then
        ask "Enter new SSH Port (e.g., 2222):" SSH_PORT
        if [[ "$SSH_PORT" =~ ^[0-9]+$ ]]; then
            sed -i "s/#Port 22/Port $SSH_PORT/" /etc/ssh/sshd_config
            sed -i "s/Port 22/Port $SSH_PORT/" /etc/ssh/sshd_config
            ufw allow "$SSH_PORT/tcp" >/dev/null
            if [ "$SSH_PORT" != "22" ]; then
                ufw delete allow 22/tcp >/dev/null 2>&1 || true
            fi
            msg "SSH Port changed to $SSH_PORT."
        else
            warn "Invalid port."
        fi
    fi
    
    systemctl restart sshd
    success "SSH Hardened"
}

show_dns_records() {
    local ip=$(curl -s https://api.ipify.org)
    msg "REQUIRED DNS RECORDS FOR $DOMAIN"
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "TYPE  | HOST                 | VALUE" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "A     | @                    | $ip" >&3
    echo -e "CNAME | www                  | $DOMAIN" >&3
    echo -e "CNAME | admin                | $DOMAIN" >&3

    [ -d "/opt/gitea" ] && echo -e "CNAME | git                  | $DOMAIN" >&3
    [ -d "/opt/nextcloud" ] && echo -e "CNAME | cloud                | $DOMAIN" >&3
    [ -d "/opt/mail" ] && echo -e "CNAME | mail                 | $DOMAIN" >&3
    [ -d "/opt/yourls" ] && echo -e "CNAME | x                    | $DOMAIN" >&3
    [ -d "/opt/vaultwarden" ] && echo -e "CNAME | pass                 | $DOMAIN" >&3
    [ -d "/opt/uptimekuma" ] && echo -e "CNAME | status               | $DOMAIN" >&3
    [ -d "/opt/wireguard" ] && echo -e "CNAME | vpn                  | $DOMAIN" >&3
    [ -d "/opt/filebrowser" ] && echo -e "CNAME | files                | $DOMAIN" >&3
    docker ps | grep -q portainer && echo -e "CNAME | portainer            | $DOMAIN" >&3
    docker ps | grep -q netdata && echo -e "CNAME | netdata              | $DOMAIN" >&3

    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "MX    | @                    | mail.$DOMAIN (Priority 10)" >&3
    echo -e "TXT   | @                    | v=spf1 mx ~all" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    ask "Press Enter to continue..." dummy
}

show_credentials() {
    clear >&3
    msg "SAVED CREDENTIALS"
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    
    if [ -f "$AUTH_FILE" ]; then
        cat "$AUTH_FILE" | while read line; do
             echo -e "${CYAN}$line${NC}" >&3
        done
    else
        echo "No credentials saved yet." >&3
    fi
    
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "Credentials file: $AUTH_FILE" >&3
    ask "Press Enter to continue..." dummy
}

show_menu() {
    clear >&3
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "${PURPLE}  CYL.AE SERVER MANAGER - V8.0${NC}" >&3
    echo -e "${PURPLE}=================================================================${NC}" >&3
    echo -e "Domain: $DOMAIN" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "1.  Gitea (Git Service)" >&3
    echo -e "2.  Nextcloud (Cloud Storage)" >&3
    echo -e "3.  Portainer (Docker UI)" >&3
    echo -e "4.  Netdata (Monitoring)" >&3
    echo -e "5.  Mail Server" >&3
    echo -e "6.  YOURLS (URL Shortener)" >&3
    echo -e "7.  FTP Server (vsftpd)" >&3
    echo -e "8.  Vaultwarden (Password Manager)" >&3
    echo -e "9.  Uptime Kuma (Status Page)" >&3
    echo -e "10. WireGuard (VPN)" >&3
    echo -e "11. FileBrowser" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "s.  System Update" >&3
    echo -e "b.  Backup Data" >&3
    echo -e "r.  Sync Infrastructure (Nginx/SSL)" >&3
    echo -e "t.  Tune System (High/Low Profile)" >&3
    echo -e "d.  Show DNS Records" >&3
    echo -e "c.  Show Credentials" >&3
    echo -e "h.  Harden SSH" >&3
    echo -e "0.  Exit" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
}

sync_infrastructure() {
    msg "Syncing Infrastructure..."

    # Reload Nginx
    if command -v nginx >/dev/null; then
        systemctl reload nginx && success "Nginx Reloaded"
    fi

    # Check Certbot renewals
    if command -v certbot >/dev/null; then
        msg "Checking SSL Renewals..."
        certbot renew --quiet
    fi

    success "Infrastructure Synced"
}

# ------------------------------------------------------------------------------
# 8. ENTRY POINT
# ------------------------------------------------------------------------------

check_root
check_os
load_config
init_system

while true; do
    show_menu
    ask "Select >" choice
    
    case $choice in
        1) [ -d "/opt/gitea" ] && manage_gitea "remove" || manage_gitea "install" ;;
        2) [ -d "/opt/nextcloud" ] && manage_nextcloud "remove" || manage_nextcloud "install" ;;
        3) docker ps | grep -q portainer && manage_portainer "remove" || manage_portainer "install" ;;
        4) docker ps | grep -q netdata && manage_netdata "remove" || manage_netdata "install" ;;
        5) [ -d "/opt/mail" ] && manage_mail "remove" || manage_mail "install" ;;
        6) [ -d "/opt/yourls" ] && manage_yourls "remove" || manage_yourls "install" ;;
        7) command -v vsftpd &>/dev/null && manage_ftp "remove" || manage_ftp "install" ;;
        8) [ -d "/opt/vaultwarden" ] && manage_vaultwarden "remove" || manage_vaultwarden "install" ;;
        9) [ -d "/opt/uptimekuma" ] && manage_uptimekuma "remove" || manage_uptimekuma "install" ;;
        10) [ -d "/opt/wireguard" ] && manage_wireguard "remove" || manage_wireguard "install" ;;
        11) [ -d "/opt/filebrowser" ] && manage_filebrowser "remove" || manage_filebrowser "install" ;;

        s) system_update ;;
        b) manage_backup ;;
        r) sync_infrastructure ;;
        t) tune_system ;;
        d) show_dns_records ;;
        c) show_credentials ;;
        h) manage_ssh ;;
        0) echo "Bye!" >&3; exit 0 ;;
        *) echo "Invalid option" >&3 ;;
    esac
    
    # Auto-sync logic for services
    if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -le 11 ] && [ "$choice" -ge 1 ]; then
        ask "Apply changes now (Update SSL/Nginx)? (y/n):" confirm
        if [[ "$confirm" == "y" ]]; then sync_infrastructure; fi
    else
        if [ "$choice" != "0" ]; then
            ask "Press Enter to continue..." dummy
        fi
    fi
done
