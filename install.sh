#!/bin/bash
# ==============================================================================
#  CYL.AE SERVER MANAGER V7.0 (Ultimate Edition)
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

# Versions
ADMINER_VER="4.8.1"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Redirect all output to log file, but keep fd 3 for user interaction
exec 3>&1 1>>"$LOG_FILE" 2>&1

# ------------------------------------------------------------------------------
# 2. UTILITY FUNCTIONS
# ------------------------------------------------------------------------------

log() { echo -e "$(date +'%Y-%m-%d %H:%M:%S') - $1"; }
msg() { echo -e "${CYAN}➜ $1${NC}" >&3; log "INFO: $1"; }
success() { echo -e "${GREEN}✔ $1${NC}" >&3; log "SUCCESS: $1"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}" >&3; log "WARN: $1"; }
error() { echo -e "${RED}✖ $1${NC}" >&3; log "ERROR: $1"; }
fatal() { error "$1"; exit 1; }

# Helper for prompts to ensure they go to fd 3 (user) not log
ask() {
    local prompt="$1"
    local var_name="$2"
    echo -ne "${YELLOW}$prompt${NC} " >&3
    read -r "$var_name"
}

check_root() {
    [[ $EUID -ne 0 ]] && fatal "This script must be run as root."
}

check_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" != "debian" && "$ID" != "ubuntu" ]]; then
             warn "Detected OS: $ID. This script is optimized for Debian/Ubuntu."
             ask "Continue anyway? (y/n):" confirm
             if [[ "$confirm" != "y" ]]; then exit 1; fi
        fi
    else
        warn "Cannot detect OS. Proceed with caution."
    fi
}

load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
    fi

    # Interactive Setup if missing domain
    if [ -z "${DOMAIN:-}" ]; then
        clear >&3
        echo -e "${PURPLE}=================================================================${NC}" >&3
        echo -e "${PURPLE}  CYL.AE SERVER MANAGER - INITIAL SETUP${NC}" >&3
        echo -e "${PURPLE}=================================================================${NC}" >&3
        echo -e "Welcome! Let's configure your server." >&3
        echo "" >&3

        while [ -z "${DOMAIN:-}" ]; do
            ask "Enter your domain name (e.g., example.com):" INPUT_DOMAIN
            if [[ "$INPUT_DOMAIN" =~ ^[a-zA-Z0-9]+([-.][a-zA-Z0-9]+)*\.[a-zA-Z]{2,}$ ]]; then
                DOMAIN="$INPUT_DOMAIN"
            else
                echo "Error: Invalid domain format." >&3
            fi
        done

        # Save to config
        echo "DOMAIN=\"$DOMAIN\"" >> "$CONFIG_FILE"
        echo "EMAIL=\"admin@$DOMAIN\"" >> "$CONFIG_FILE"
        echo "INSTALL_DIR=\"$(pwd)\"" >> "$CONFIG_FILE"
        chmod 600 "$CONFIG_FILE"
    fi

    EMAIL="admin@$DOMAIN"
}

generate_password() {
    openssl rand -base64 16 | tr -dc 'a-zA-Z0-9' | head -c 16
}

# ------------------------------------------------------------------------------
# 3. SYSTEM INFRASTRUCTURE
# ------------------------------------------------------------------------------

detect_profile() {
    TOTAL_RAM=$(free -m | awk '/Mem:/ { print $2 }')
    msg "Detected RAM: ${TOTAL_RAM}MB"
    
    if [ "$TOTAL_RAM" -lt 3800 ]; then
        PROFILE="LOW"
        msg "Profile selected: LOW RESOURCE (Optimization for stability)"
        echo "LOW" > /etc/cyl_profile
    else
        PROFILE="HIGH"
        msg "Profile selected: HIGH PERFORMANCE (Optimization for speed)"
        echo "HIGH" > /etc/cyl_profile
    fi
}

tune_system() {
    msg "Applying System Tuning..."
    PROFILE=$(cat /etc/cyl_profile 2>/dev/null || echo "HIGH")

    # 1. Database Tuning (MariaDB)
    # Check if MariaDB is installed first
    if [ -d "/etc/mysql/mariadb.conf.d" ]; then
        if [ "$PROFILE" == "LOW" ]; then
            cat <<EOF > /etc/mysql/mariadb.conf.d/99-tuning.cnf
[mysqld]
innodb_buffer_pool_size = 128M
max_connections = 50
performance_schema = OFF
bind-address = 0.0.0.0
EOF
        else
            cat <<EOF > /etc/mysql/mariadb.conf.d/99-tuning.cnf
[mysqld]
innodb_buffer_pool_size = 1G
max_connections = 200
bind-address = 0.0.0.0
EOF
        fi
        systemctl restart mariadb || warn "Failed to restart MariaDB"
    fi

    # 2. PHP Tuning
    if command -v php >/dev/null; then
        PHP_VER=$(php -r "echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;")
        PHP_FPM_CONF="/etc/php/$PHP_VER/fpm/pool.d/www.conf"
        PHP_INI="/etc/php/$PHP_VER/fpm/php.ini"

        if [ -f "$PHP_FPM_CONF" ]; then
            if [ "$PROFILE" == "LOW" ]; then
                sed -i 's/^pm.max_children = .*/pm.max_children = 10/' "$PHP_FPM_CONF"
                sed -i 's/^memory_limit = .*/memory_limit = 256M/' "$PHP_INI"
            else
                sed -i 's/^pm.max_children = .*/pm.max_children = 50/' "$PHP_FPM_CONF"
                sed -i 's/^memory_limit = .*/memory_limit = 512M/' "$PHP_INI"
            fi
            # Common
            sed -i 's/^upload_max_filesize = .*/upload_max_filesize = 512M/' "$PHP_INI"
            sed -i 's/^post_max_size = .*/post_max_size = 512M/' "$PHP_INI"

            systemctl restart "php$PHP_VER-fpm" || warn "Failed to restart PHP-FPM"
        fi
    fi

    success "System Tuned for $PROFILE usage."
}

init_system() {
    if [ -f "/root/.server_installed" ]; then
        return
    fi
    msg "Initializing System Infrastructure..."
    
    export DEBIAN_FRONTEND=noninteractive

    # 1. Basics
    if ! command -v jq &> /dev/null; then
        msg "Installing Basic Dependencies..."
        apt-get update -q && apt-get install -y curl wget git unzip gnupg2 apt-transport-https ca-certificates lsb-release ufw sudo htop apache2-utils fail2ban jq bc >/dev/null
    fi
    
    # 2. Swap & BBR
    if [ ! -f /swapfile ]; then
        msg "Creating Swap File..."
        fallocate -l 2G /swapfile || dd if=/dev/zero of=/swapfile bs=1G count=2
        chmod 600 /swapfile
        mkswap /swapfile
        swapon /swapfile
        echo '/swapfile none swap sw 0 0' >> /etc/fstab
    fi
    
    if ! grep -q "tcp_bbr" /etc/sysctl.conf; then
        echo "net.core.default_qdisc=fq" >> /etc/sysctl.conf
        echo "net.ipv4.tcp_congestion_control=bbr" >> /etc/sysctl.conf
        sysctl -p >/dev/null
    fi
    
    # 3. DNS Optimization
    if [ -f /etc/systemd/resolved.conf ]; then
        sed -i 's/#DNS=/DNS=8.8.8.8 8.8.4.4 1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf
        sed -i 's/#FallbackDNS=/FallbackDNS=1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf
        systemctl restart systemd-resolved || true
    fi

    # 4. Docker Engine (Official)
    if ! command -v docker &> /dev/null; then
        msg "Installing Docker..."
        mkdir -p /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
        apt-get update -q && apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin >/dev/null
    fi
    
    if ! docker network inspect $DOCKER_NET >/dev/null 2>&1; then
        docker network create $DOCKER_NET
    fi

    # 5. Host Stack (Nginx/PHP/MariaDB/Certbot)
    msg "Installing Host Stack..."
    apt-get install -y nginx mariadb-server certbot python3-certbot-nginx php-fpm php-mysql php-curl php-gd php-mbstring php-xml php-zip php-intl php-bcmath >/dev/null
    
    detect_profile
    tune_system
    
    # 6. Firewall
    ufw allow ssh >/dev/null
    ufw allow http >/dev/null
    ufw allow https >/dev/null

    # Allow Docker subnet to access Host MariaDB
    SUBNET=$(docker network inspect $DOCKER_NET | jq -r '.[0].IPAM.Config[0].Subnet')
    ufw allow from "$SUBNET" to any port 3306 >/dev/null
    echo "y" | ufw enable >/dev/null

    # 7. Auth Init
    if [ ! -f $AUTH_FILE ]; then touch $AUTH_FILE && chmod 600 $AUTH_FILE; fi
    init_db_password
    
    setup_autoupdate
    
    touch /root/.server_installed
    success "System Initialized"
}

init_db_password() {
    if grep -q "mysql_root_password" $AUTH_FILE; then
        DB_ROOT_PASS=$(grep "mysql_root_password" $AUTH_FILE | cut -d= -f2)
    else
        if [ -f /root/.mariadb_auth ]; then
             DB_ROOT_PASS=$(grep mysql_root_password /root/.mariadb_auth | cut -d= -f2)
        else
             DB_ROOT_PASS=$(generate_password)
             echo "mysql_root_password=$DB_ROOT_PASS" >> $AUTH_FILE
             mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED BY '$DB_ROOT_PASS'; FLUSH PRIVILEGES;" || true
        fi
    fi
    export DB_ROOT_PASS
}

ensure_db() {
    # Usage: ensure_db dbname username password
    local db=$1
    local user=$2
    local pass=$3
    mysql -u root --password="$DB_ROOT_PASS" -e "CREATE DATABASE IF NOT EXISTS \`$db\`; CREATE USER IF NOT EXISTS '$user'@'%' IDENTIFIED BY '$pass'; GRANT ALL PRIVILEGES ON \`$db\`.* TO '$user'@'%'; FLUSH PRIVILEGES;"
}

# ------------------------------------------------------------------------------
# 4. CORE MODULES (Nginx, Docker Helpers)
# ------------------------------------------------------------------------------

update_nginx() {
    local sub=$1
    local port=$2
    local type=$3 # proxy or php
    local root=${4:-}
    
    # Security Headers
    local SEC_HEADERS='
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    '

    rm -f "/etc/nginx/sites-enabled/default"

    cat <<EOF > "/etc/nginx/sites-available/$sub"
server {
    listen 80;
    server_name $sub;
    client_max_body_size 10G;
    $SEC_HEADERS
EOF

    if [ "$type" == "proxy" ]; then
        cat <<EOF >> "/etc/nginx/sites-available/$sub"
    location / {
        proxy_pass http://127.0.0.1:$port;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF
    elif [ "$type" == "php" ] || [ "$type" == "dashboard" ]; then
        PHP_SOCK=$(find /run/php/ -name "php*-fpm.sock" | head -n 1)
        cat <<EOF >> "/etc/nginx/sites-available/$sub"
    root $root;
    index index.php index.html;
EOF
        if [ "$type" == "dashboard" ]; then
            echo '    location / { try_files $uri $uri/ =404; auth_basic "Restricted"; auth_basic_user_file /etc/nginx/.htpasswd; }' >> "/etc/nginx/sites-available/$sub"
        else
             echo '    location / { try_files $uri $uri/ =404; }' >> "/etc/nginx/sites-available/$sub"
        fi

        cat <<EOF >> "/etc/nginx/sites-available/$sub"
    location ~ \.php$ {
        include snippets/fastcgi-php.conf;
        fastcgi_pass unix:$PHP_SOCK;
    }
}
EOF
    fi

    ln -sf "/etc/nginx/sites-available/$sub" "/etc/nginx/sites-enabled/"
}

# Generic Docker Service Installer
deploy_docker_service() {
    local name=$1
    local pretty_name=$2
    local subdomain=$3
    local port=$4
    local docker_compose_content=$5

    msg "Installing $pretty_name..."
    mkdir -p "/opt/$name"

    echo "$docker_compose_content" > "/opt/$name/docker-compose.yml"

    cd "/opt/$name" && docker compose up -d

    update_nginx "$subdomain" "$port" "proxy"
    success "$pretty_name Installed at https://$subdomain"
}

remove_docker_service() {
    local name=$1
    local pretty_name=$2
    local subdomain=$3
    local port=${4:-}

    msg "Removing $pretty_name..."

    ask "Do you want to PERMANENTLY DELETE all data for $pretty_name? (y/n):" confirm_delete

    if [ -d "/opt/$name" ]; then
        cd "/opt/$name" && docker compose down
        if [[ "$confirm_delete" == "y" ]]; then
            cd / && rm -rf "/opt/$name"
            warn "Data directory /opt/$name deleted."
        else
            warn "Data directory /opt/$name PRESERVED."
        fi
    fi

    rm -f "/etc/nginx/sites-enabled/$subdomain"
    if [ -n "$port" ]; then
        ufw delete allow "$port" >/dev/null 2>&1 || true
    fi
    success "$pretty_name Removed"
}

# ------------------------------------------------------------------------------
# 5. SERVICE DEFINITIONS
# ------------------------------------------------------------------------------

manage_gitea() {
    local name="gitea"
    local sub="git.$DOMAIN"
    if [ "$1" == "install" ]; then
        local pass=$(generate_password)
        ensure_db "$name" "$name" "$pass"
        local host_ip=$(hostname -I | awk '{print $1}')

        read -r -d '' CONTENT <<EOF
version: "3"
services:
  server:
    image: gitea/gitea:latest
    container_name: gitea
    networks:
      - $DOCKER_NET
    environment:
      - USER_UID=1000
      - USER_GID=1000
      - GITEA__database__DB_TYPE=mysql
      - GITEA__database__HOST=$host_ip:3306
      - GITEA__database__NAME=$name
      - GITEA__database__USER=$name
      - GITEA__database__PASSWD=$pass
      - GITEA__server__SSH_DOMAIN=$sub
      - GITEA__server__SSH_PORT=2222
      - GITEA__server__ROOT_URL=https://$sub/
    restart: always
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:3000:3000"
      - "2222:22"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Gitea" "$sub" "3000" "$CONTENT"
    else
        remove_docker_service "$name" "Gitea" "$sub"
    fi
}

manage_nextcloud() {
    local name="nextcloud"
    local sub="cloud.$DOMAIN"
    if [ "$1" == "install" ]; then
        local pass=$(generate_password)
        ensure_db "$name" "$name" "$pass"
        local host_ip=$(hostname -I | awk '{print $1}')

        read -r -d '' CONTENT <<EOF
version: '2'
services:
  app:
    image: nextcloud
    container_name: nextcloud_app
    restart: always
    networks:
      - $DOCKER_NET
    ports:
      - 127.0.0.1:8080:80
    volumes:
      - ./nextcloud:/var/www/html
    environment:
      - MYSQL_PASSWORD=$pass
      - MYSQL_DATABASE=$name
      - MYSQL_USER=$name
      - MYSQL_HOST=$host_ip
      - NEXTCLOUD_TRUSTED_DOMAINS=$sub
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Nextcloud" "$sub" "8080" "$CONTENT"

        msg "Waiting for Nextcloud to initialize..."
        until docker exec -u www-data nextcloud_app php occ status >/dev/null 2>&1; do sleep 2; done
        docker exec -u www-data nextcloud_app php occ config:system:set trusted_proxies 0 --value="127.0.0.1" >/dev/null 2>&1
        docker exec -u www-data nextcloud_app php occ config:system:set overwriteprotocol --value="https" >/dev/null 2>&1
    else
        remove_docker_service "$name" "Nextcloud" "$sub"
    fi
}

manage_vaultwarden() {
    local name="vaultwarden"
    local sub="pass.$DOMAIN"
    if [ "$1" == "install" ]; then
        read -r -d '' CONTENT <<EOF
version: '3'
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: always
    environment:
      - SIGNUPS_ALLOWED=true
    volumes:
      - ./data:/data
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8082:80"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Vaultwarden" "$sub" "8082" "$CONTENT"
    else
        remove_docker_service "$name" "Vaultwarden" "$sub"
    fi
}

manage_uptimekuma() {
    local name="uptimekuma"
    local sub="status.$DOMAIN"
    if [ "$1" == "install" ]; then
        read -r -d '' CONTENT <<EOF
version: '3'
services:
  uptime-kuma:
    image: louislam/uptime-kuma:1
    container_name: uptime-kuma
    restart: always
    volumes:
      - ./data:/app/data
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:3001:3001"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Uptime Kuma" "$sub" "3001" "$CONTENT"
    else
        remove_docker_service "$name" "Uptime Kuma" "$sub"
    fi
}

manage_wireguard() {
    local name="wireguard"
    local sub="vpn.$DOMAIN"
    if [ "$1" == "install" ]; then
        ask "Enter WG Password (leave empty for auto):" WGPASS
        if [ -z "$WGPASS" ]; then
            WGPASS=$(generate_password)
            msg "Generated WG Password: $WGPASS"
            echo "wg_pass=$WGPASS" >> $AUTH_FILE
        fi
        local host_ip=$(curl -s https://api.ipify.org)

        read -r -d '' CONTENT <<EOF
version: "3.8"
services:
  wg-easy:
    environment:
      - WG_HOST=$host_ip
      - PASSWORD=$WGPASS
      - WG_PORT=51820
      - WG_DEFAULT_ADDRESS=10.8.0.x
      - WG_DEFAULT_DNS=1.1.1.1
      - WG_ALLOWED_IPS=0.0.0.0/0
      - WG_PERSISTENT_KEEPALIVE=25
    image: ghcr.io/wg-easy/wg-easy
    container_name: wg-easy
    volumes:
      - ./data:/etc/wireguard
    ports:
      - "51820:51820/udp"
      - "127.0.0.1:51821:51821/tcp"
    restart: unless-stopped
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    sysctls:
      - net.ipv4.ip_forward=1
      - net.ipv4.conf.all.src_valid_mark=1
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "WireGuard" "$sub" "51821" "$CONTENT"
        ufw allow 51820/udp >/dev/null
        success "Credentials saved to $AUTH_FILE"
    else
        remove_docker_service "$name" "WireGuard" "$sub"
        ufw delete allow 51820/udp >/dev/null 2>&1 || true
    fi
}

manage_filebrowser() {
    local name="filebrowser"
    local sub="files.$DOMAIN"
    if [ "$1" == "install" ]; then
        mkdir -p /opt/$name
        touch /opt/$name/filebrowser.db
        echo '{"port": 80, "baseURL": "", "address": "", "log": "stdout", "database": "/database.db", "root": "/srv"}' > /opt/$name/settings.json

        read -r -d '' CONTENT <<EOF
version: '3'
services:
  filebrowser:
    image: filebrowser/filebrowser
    container_name: filebrowser
    restart: always
    volumes:
      - /var/www:/srv
      - ./filebrowser.db:/database.db
      - ./settings.json:/.filebrowser.json
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8083:80"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "FileBrowser" "$sub" "8083" "$CONTENT"
        success "Default login: admin/admin"
    else
        remove_docker_service "$name" "FileBrowser" "$sub"
    fi
}

manage_yourls() {
    local name="yourls"
    local sub="x.$DOMAIN"
    if [ "$1" == "install" ]; then
        local pass=$(generate_password)
        ensure_db "$name" "$name" "$pass"
        local host_ip=$(hostname -I | awk '{print $1}')
        local cookie=$(openssl rand -hex 16)

        # NOTE: Mapping /var/www/html to local 'data' dir to persist plugins/config
        read -r -d '' CONTENT <<EOF
version: '3'
services:
  yourls:
    image: yourls:latest
    container_name: yourls
    restart: always
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8084:80"
    volumes:
      - ./data:/var/www/html
    environment:
      - YOURLS_DB_HOST=$host_ip
      - YOURLS_DB_USER=$name
      - YOURLS_DB_PASS=$pass
      - YOURLS_DB_NAME=$name
      - YOURLS_SITE=https://$sub
      - YOURLS_USER=admin
      - YOURLS_PASS=$pass
      - YOURLS_COOKIEKEY=$cookie
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "YOURLS" "$sub" "8084" "$CONTENT"
        echo "yourls_pass=$pass" >> $AUTH_FILE
        success "Credentials saved to $AUTH_FILE"
    else
        remove_docker_service "$name" "YOURLS" "$sub"
    fi
}

manage_mail() {
    local name="mail"
    local sub="mail.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing Mail Server..."
        mkdir -p /opt/mail
        
        # Resource Check
        local profile=$(cat /etc/cyl_profile)
        local clamav_state=1
        if [ "$profile" == "LOW" ]; then
            clamav_state=0
            msg "Low RAM detected: Disabling ClamAV."
        fi
        
        read -r -d '' CONTENT <<EOF
version: '3'
services:
  mailserver:
    image: mailserver/docker-mailserver:latest
    hostname: mail
    domainname: $DOMAIN
    container_name: mailserver
    networks:
      - $DOCKER_NET
    ports:
      - "25:25"
      - "143:143"
      - "587:587"
      - "993:993"
    volumes:
      - ./maildata:/var/mail
      - ./mailstate:/var/mail-state
      - ./maillogs:/var/log/mail
      - ./config:/tmp/docker-mailserver
    environment:
      - ENABLE_SPAMASSASSIN=1
      - ENABLE_CLAMAV=$clamav_state
      - ENABLE_FAIL2BAN=1
      - SSL_TYPE=letsencrypt
    cap_add:
      - NET_ADMIN
    restart: always
  roundcube:
    image: roundcube/roundcubemail:latest
    container_name: roundcube
    networks:
      - $DOCKER_NET
    restart: always
    volumes:
      - ./roundcube/db:/var/www/db
    ports:
      - "127.0.0.1:8081:80"
    environment:
      - ROUNDCUBEMAIL_DEFAULT_HOST=mailserver
      - ROUNDCUBEMAIL_SMTP_SERVER=mailserver
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Mail Server" "$sub" "8081" "$CONTENT"

        ufw allow 25,587,465,143,993/tcp >/dev/null
        
        msg "Initializing Mail User..."
        until docker exec mailserver setup email list >/dev/null 2>&1; do sleep 2; done
        
        if ! docker exec mailserver setup email list | grep -q "postmaster"; then
            local pass=$(generate_password)
            docker exec mailserver setup email add postmaster@$DOMAIN "$pass"
            docker exec mailserver setup config dkim
            echo "postmaster_pass=$pass" >> $AUTH_FILE
            msg "Created default admin: postmaster@$DOMAIN"
        fi
        success "Credentials saved to $AUTH_FILE"
    else
        remove_docker_service "$name" "Mail Server" "$sub"
        ufw delete allow 25,587,465,143,993/tcp >/dev/null 2>&1 || true
    fi
}

manage_netdata() {
    local name="netdata"
    local sub="netdata.$DOMAIN"

    if [ "$1" == "install" ]; then
        msg "Installing Netdata..."
        mkdir -p "/opt/$name"
        # Using bind mounts to ensure we can back them up
        mkdir -p "/opt/$name/config" "/opt/$name/lib" "/opt/$name/cache"

        # We run this one manually as it requires many host mounts
        docker run -d --name=netdata --pid=host --network=$DOCKER_NET -p 127.0.0.1:19999:19999 \
          -v "/opt/$name/config:/etc/netdata" \
          -v "/opt/$name/lib:/var/lib/netdata" \
          -v "/opt/$name/cache:/var/cache/netdata" \
          -v /etc/passwd:/host/etc/passwd:ro -v /etc/group:/host/etc/group:ro -v /proc:/host/proc:ro \
          -v /sys:/host/sys:ro -v /etc/os-release:/host/etc/os-release:ro -v /var/run/docker.sock:/var/run/docker.sock \
          --restart always --cap-add SYS_PTRACE --security-opt apparmor=unconfined netdata/netdata

        update_nginx "$sub" "19999" "proxy"
        success "Netdata Installed"
    elif [ "$1" == "remove" ]; then
        docker stop netdata && docker rm netdata

        ask "Do you want to PERMANENTLY DELETE data for Netdata? (y/n):" confirm_delete
        if [[ "$confirm_delete" == "y" ]]; then
            rm -rf "/opt/$name"
        fi

        rm -f "/etc/nginx/sites-enabled/$sub"
        success "Netdata Removed"
    fi
}

manage_portainer() {
    local name="portainer"
    local sub="portainer.$DOMAIN"

    if [ "$1" == "install" ]; then
        msg "Installing Portainer..."
        mkdir -p "/opt/$name/data"

        docker run -d -p 127.0.0.1:9000:9000 --name=portainer --network $DOCKER_NET --restart=always \
        -v /var/run/docker.sock:/var/run/docker.sock -v "/opt/$name/data:/data" portainer/portainer-ce

        update_nginx "$sub" "9000" "proxy"
        success "Portainer Installed"
    elif [ "$1" == "remove" ]; then
        docker stop portainer && docker rm portainer

        ask "Do you want to PERMANENTLY DELETE data for Portainer? (y/n):" confirm_delete
        if [[ "$confirm_delete" == "y" ]]; then
            rm -rf "/opt/$name"
        fi

        rm -f "/etc/nginx/sites-enabled/$sub"
        success "Portainer Removed"
    fi
}

manage_ftp() {
    if [ "$1" == "install" ]; then
        msg "Installing FTP (vsftpd)..."
        apt-get install -y vsftpd >/dev/null
        cp /etc/vsftpd.conf /etc/vsftpd.conf.bak 2>/dev/null

        local pasv_addr=$(curl -s https://api.ipify.org)
        cat <<EOF > /etc/vsftpd.conf
listen=NO
listen_ipv6=YES
anonymous_enable=NO
local_enable=YES
write_enable=YES
dirmessage_enable=YES
use_localtime=YES
xferlog_enable=YES
connect_from_port_20=YES
chroot_local_user=YES
secure_chroot_dir=/var/run/vsftpd/empty
pam_service_name=vsftpd
rsa_cert_file=/etc/ssl/certs/ssl-cert-snakeoil.pem
rsa_private_key=/etc/ssl/private/ssl-cert-snakeoil.key
ssl_enable=YES
allow_anon_ssl=NO
ssl_tlsv1=YES
pasv_min_port=40000
pasv_max_port=50000
allow_writeable_chroot=YES
pasv_address=$pasv_addr
EOF
        ufw allow 20,21,990/tcp >/dev/null
        ufw allow 40000:50000/tcp >/dev/null
        systemctl restart vsftpd
        
        if ! id "cyluser" &>/dev/null; then
            local ftp_pass=$(generate_password)
            useradd -m -d /var/www -s /bin/bash cyluser
            echo "cyluser:$ftp_pass" | chpasswd
            usermod -a -G www-data cyluser
            chown -R cyluser:www-data /var/www
            echo "ftp_user=cyluser" >> $AUTH_FILE
            echo "ftp_pass=$ftp_pass" >> $AUTH_FILE
        fi
        success "FTP Installed"
    elif [ "$1" == "remove" ]; then
        apt-get remove -y vsftpd >/dev/null
        ufw delete allow 20/tcp >/dev/null 2>&1 || true
        success "FTP Removed"
    fi
}

# ------------------------------------------------------------------------------
# 6. HOUSEKEEPING & EXTRAS
# ------------------------------------------------------------------------------

manage_backup() {
    msg "Starting Backup..."
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    
    # DB Dump
    mysqldump -u root --password="$DB_ROOT_PASS" --all-databases > "$BACKUP_DIR/db_$TIMESTAMP.sql"
    
    # Files
    # Note: Portainer and Netdata now use bind mounts in /opt/, so they are included.
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

setup_autoupdate() {
    msg "Configuring Auto-Update..."
    cp "$(realpath "$0")" /usr/local/bin/server_manager.sh
    cp "$(dirname "$0")/auto_update.sh" /usr/local/bin/server_autoupdate.sh
    chmod +x /usr/local/bin/server_manager.sh
    chmod +x /usr/local/bin/server_autoupdate.sh
    
    CRON_CMD="0 4 * * * /usr/local/bin/server_autoupdate.sh"
    (crontab -l 2>/dev/null | grep -v "server_autoupdate.sh"; echo "$CRON_CMD") | crontab -
    
    success "Auto-Update Scheduled (Daily @ 04:00)"
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
    msg "Retrieving Credentials..."
    if [ -f "$AUTH_FILE" ]; then
        echo -e "${YELLOW}=== CREDENTIALS ($AUTH_FILE) ===${NC}" >&3
        cat "$AUTH_FILE" >&3
        echo -e "${YELLOW}========================================${NC}" >&3
    else
        warn "No credentials file found at $AUTH_FILE"
    fi
    ask "Press Enter to continue..." dummy
}

# ------------------------------------------------------------------------------
# 7. SYNC & UI
# ------------------------------------------------------------------------------

sync_infrastructure() {
    msg "Syncing Dashboard & SSL..."
    mkdir -p /var/www/dashboard
    if ! grep -q "dashboard_pass" $AUTH_FILE; then
        PASS=$(generate_password)
        echo "dashboard_user=admin" >> $AUTH_FILE
        echo "dashboard_pass=$PASS" >> $AUTH_FILE
        htpasswd -bc /etc/nginx/.htpasswd admin "$PASS"
    fi
    
    # Generate Links
    local LINKS="<a href='/adminer.php' class='card' target='_blank'>DB Admin</a>"
    [ -d "/opt/gitea" ] && LINKS+="<a href='https://git.$DOMAIN' class='card'>Gitea</a>"
    [ -d "/opt/nextcloud" ] && LINKS+="<a href='https://cloud.$DOMAIN' class='card'>Nextcloud</a>"
    [ -d "/opt/mail" ] && LINKS+="<a href='https://mail.$DOMAIN' class='card'>Webmail</a>"
    [ -d "/opt/yourls" ] && LINKS+="<a href='https://x.$DOMAIN' class='card'>Shortener</a>"
    [ -d "/opt/vaultwarden" ] && LINKS+="<a href='https://pass.$DOMAIN' class='card'>Vaultwarden</a>"
    [ -d "/opt/uptimekuma" ] && LINKS+="<a href='https://status.$DOMAIN' class='card'>Status</a>"
    [ -d "/opt/wireguard" ] && LINKS+="<a href='https://vpn.$DOMAIN' class='card'>VPN</a>"
    [ -d "/opt/filebrowser" ] && LINKS+="<a href='https://files.$DOMAIN' class='card'>Files</a>"
    docker ps | grep -q portainer && LINKS+="<a href='https://portainer.$DOMAIN' class='card'>Portainer</a>"
    docker ps | grep -q netdata && LINKS+="<a href='https://netdata.$DOMAIN' class='card'>Monitoring</a>"

    # Dashboard HTML
    cat <<EOF > /var/www/dashboard/index.php
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Admin $DOMAIN</title>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;500;700&display=swap" rel="stylesheet">
    <style>
        :root { --primary: #00d2ff; --bg: #0f172a; --card: #1e293b; --text: #f8fafc; }
        body { font-family: 'Outfit', sans-serif; background: var(--bg); color: var(--text); margin: 0; min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; background-image: radial-gradient(circle at top right, #1e293b 0%, #0f172a 100%); }
        h1 { font-weight: 700; font-size: 2.5rem; margin-bottom: 10px; background: linear-gradient(45deg, #fff, var(--primary)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
        p { color: #94a3b8; margin-bottom: 40px; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; width: 90%; max-width: 1000px; }
        .card { background: rgba(30, 41, 59, 0.7); backdrop-filter: blur(10px); padding: 30px; border-radius: 16px; border: 1px solid rgba(255,255,255,0.1); text-decoration: none; color: var(--text); font-weight: 500; transition: all 0.3s ease; text-align: center; display: flex; align-items: center; justify-content: center; flex-direction: column; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); }
        .card:hover { transform: translateY(-5px); border-color: var(--primary); box-shadow: 0 20px 25px -5px rgba(0, 210, 255, 0.15); color: var(--primary); }
    </style>
</head>
<body>
    <h1>Server Dashboard</h1>
    <p>System Profile: $(cat /etc/cyl_profile) | <a href='/adminer.php' style='color:var(--primary)'>DB Admin</a></p>
    <div class='grid'>$LINKS</div>
</body>
</html>
EOF
    # Download Adminer
    if [ ! -f /var/www/dashboard/adminer.php ]; then
         wget -q "https://github.com/vrana/adminer/releases/download/v$ADMINER_VER/adminer-$ADMINER_VER.php" -O /var/www/dashboard/adminer.php
    fi
    chown -R www-data:www-data /var/www/dashboard
    
    update_nginx "admin.$DOMAIN" "0" "dashboard" "/var/www/dashboard"
    
    # Landing Page
    mkdir -p /var/www/landing
    echo "<!DOCTYPE html><html><head><title>Welcome to $DOMAIN</title><style>body{background:#000;color:#fff;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;font-family:sans-serif}h1{font-weight:300;letter-spacing:5px}</style></head><body><h1>$DOMAIN</h1></body></html>" > /var/www/landing/index.html
    chown -R www-data:www-data /var/www/landing
    update_nginx "$DOMAIN" "0" "php" "/var/www/landing"

    nginx -t && systemctl reload nginx
    
    # Certbot Expansion
    local DOMAINS="-d $DOMAIN -d admin.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/git.$DOMAIN ] && DOMAINS+=" -d git.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/cloud.$DOMAIN ] && DOMAINS+=" -d cloud.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/mail.$DOMAIN ] && DOMAINS+=" -d mail.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/x.$DOMAIN ] && DOMAINS+=" -d x.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/pass.$DOMAIN ] && DOMAINS+=" -d pass.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/status.$DOMAIN ] && DOMAINS+=" -d status.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/vpn.$DOMAIN ] && DOMAINS+=" -d vpn.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/files.$DOMAIN ] && DOMAINS+=" -d files.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/portainer.$DOMAIN ] && DOMAINS+=" -d portainer.$DOMAIN"
    [ -f /etc/nginx/sites-enabled/netdata.$DOMAIN ] && DOMAINS+=" -d netdata.$DOMAIN"
    
    echo "Updating SSL..."
    certbot --nginx $DOMAINS --non-interactive --agree-tos -m "$EMAIL" --redirect --expand
    
    success "Infrastructure Synced."
}

show_menu() {
    clear >&3
    PROFILE=$(cat /etc/cyl_profile 2>/dev/null || echo "DETECTING...")
    
    # Health Stats
    LOAD=$(uptime | awk -F'load average:' '{ print $2 }' | cut -d, -f1 | xargs)
    DISK=$(df -h / | awk 'NR==2 {print $5}')
    RAM=$(free -m | awk '/Mem:/ {printf "%.0f%%", $3/$2*100}')
    
    echo -e "${PURPLE}=== CYL.AE MANAGER V7.0 - $DOMAIN [$PROFILE] ===${NC}" >&3
    echo -e "${BLUE}Health: Load: $LOAD | Disk: $DISK | RAM: $RAM${NC}" >&3
    echo -e "-----------------------------" >&3
    
    # Helper for status color
    is_active() {
        if [ "$1" == "dir" ]; then [ -d "$2" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}"; fi
        if [ "$1" == "docker" ]; then docker ps | grep -q "$2" && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}"; fi
        if [ "$1" == "cmd" ]; then command -v "$2" &>/dev/null && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}"; fi
    }

    echo -e "1. Gitea       [$(is_active dir /opt/gitea)]" >&3
    echo -e "2. Nextcloud   [$(is_active dir /opt/nextcloud)]" >&3
    echo -e "3. Portainer   [$(is_active docker portainer)]" >&3
    echo -e "4. Netdata     [$(is_active docker netdata)]" >&3
    echo -e "5. Mail Server [$(is_active dir /opt/mail)]" >&3
    echo -e "6. YOURLS      [$(is_active dir /opt/yourls)]" >&3
    echo -e "7. FTP Server  [$(is_active cmd vsftpd)]" >&3
    echo -e "8. Vaultwarden [$(is_active dir /opt/vaultwarden)]" >&3
    echo -e "9. Uptime Kuma [$(is_active dir /opt/uptimekuma)]" >&3
    echo -e "10. WireGuard  [$(is_active dir /opt/wireguard)]" >&3
    echo -e "11. FileBrowse [$(is_active dir /opt/filebrowser)]" >&3
    echo -e "-----------------------------" >&3
    echo -e "s. SYSTEM UPDATE" >&3
    echo -e "b. BACKUP NOW" >&3
    echo -e "r. SYNC ALL (SSL & Dashboard)" >&3
    echo -e "t. RE-TUNE SYSTEM" >&3
    echo -e "d. SHOW DNS RECORDS" >&3
    echo -e "c. SHOW CREDENTIALS" >&3
    echo -e "h. HARDEN SSH" >&3
    echo -e "0. Exit" >&3
    echo -e "" >&3
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
