#!/bin/bash

export DEBIAN_FRONTEND=noninteractive

# ==============================================================================
# ==============================================================================
#  CYL.AE SERVER MANAGER V6.0 (Universal Edition)
#  Features: Auto-Tuning, Modular, Monitoring, SSL-Sync
# ==============================================================================

# ------------------------------------------------------------------------------
# 1. CONFIGURATION
# ------------------------------------------------------------------------------

CONFIG_FILE="/etc/cyl_manager.conf"

# Load Config
if [ -f "$CONFIG_FILE" ]; then
    source "$CONFIG_FILE"
fi

# Initial Setup Prompt
if [ -z "$DOMAIN" ]; then
    clear
    echo "================================================================="
    echo "  CYL.AE SERVER MANAGER - INITIAL SETUP"
    echo "================================================================="
    echo "Welcome! Let's configure your server."
    echo ""
    read -p "Enter your domain name (e.g., example.com): " DOMAIN
    
    if [ -z "$DOMAIN" ]; then
        echo "Error: Domain cannot be empty."
        exit 1
    fi

    if [[ ! "$DOMAIN" =~ ^[a-zA-Z0-9]+([-.][a-zA-Z0-9]+)*\.[a-zA-Z]{2,}$ ]]; then
        echo "Error: Invalid domain format (e.g., example.com)."
        exit 1
    fi
    
    # Save to config
    echo "DOMAIN=\"$DOMAIN\"" >> "$CONFIG_FILE"
    echo "EMAIL=\"admin@$DOMAIN\"" >> "$CONFIG_FILE"
    echo "INSTALL_DIR=\"$(pwd)\"" >> "$CONFIG_FILE"
fi

EMAIL="admin@$DOMAIN"
LOG_FILE="/var/log/server_manager.log"
AUTH_FILE="/root/.auth_details"
DOCKER_NET="server-net"
BACKUP_DIR="/var/backups/cyl_manager"
ADMINER_VER="4.8.1"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

exec 3>&1 1>>$LOG_FILE 2>&1

log() { echo -e "$(date +'%Y-%m-%d %H:%M:%S') - $1"; }
msg() { echo -e "${CYAN}➜ $1${NC}" >&3; log "$1"; }
success() { echo -e "${GREEN}✔ $1${NC}" >&3; log "SUCCESS: $1"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}" >&3; log "WARN: $1"; }

# Check Root
[[ $EUID -ne 0 ]] && echo "Run as root" >&3 && exit 1

# ------------------------------------------------------------------------------
# 2. INTELLIGENT RESOURCE DETECTION
# ------------------------------------------------------------------------------

detect_profile() {
    TOTAL_RAM=$(free -m | awk '/Mem:/ { print $2 }')
    msg "Detected RAM: ${TOTAL_RAM}MB"
    
    if [ "$TOTAL_RAM" -lt 3800 ]; then
        PROFILE="LOW"
        msg "Profile selected: LOW RESOURCE (Optimization for stability)"
        echo "LOW" > /tmp/server_profile
    else
        PROFILE="HIGH"
        msg "Profile selected: HIGH PERFORMANCE (Optimization for speed)"
        echo "HIGH" > /tmp/server_profile
    fi
}

# ------------------------------------------------------------------------------
# 3. SYSTEM INFRASTRUCTURE (The Foundation)
# ------------------------------------------------------------------------------

tune_system() {
    msg "Applying System Tuning..."
    PROFILE=$(cat /tmp/server_profile)

    # 1. Database Tuning
    if [ "$PROFILE" == "LOW" ]; then
        # Low RAM Optimization
        cat <<EOF > /etc/mysql/mariadb.conf.d/99-tuning.cnf
[mysqld]
innodb_buffer_pool_size = 128M
max_connections = 50
performance_schema = OFF
bind-address = 0.0.0.0
EOF
    else
        # High RAM Optimization
        cat <<EOF > /etc/mysql/mariadb.conf.d/99-tuning.cnf
[mysqld]
innodb_buffer_pool_size = 1G
max_connections = 200
bind-address = 0.0.0.0
EOF
    fi

    # 2. PHP Tuning
    PHP_VER=$(php -r "echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;")
    if [ "$PROFILE" == "LOW" ]; then
        sed -i 's/pm.max_children = .*/pm.max_children = 10/' /etc/php/$PHP_VER/fpm/pool.d/www.conf
        sed -i 's/memory_limit = .*/memory_limit = 256M/' /etc/php/$PHP_VER/fpm/php.ini
    else
        sed -i 's/pm.max_children = .*/pm.max_children = 50/' /etc/php/$PHP_VER/fpm/pool.d/www.conf
        sed -i 's/memory_limit = .*/memory_limit = 512M/' /etc/php/$PHP_VER/fpm/php.ini
    fi
    # Common PHP settings
    sed -i 's/upload_max_filesize = .*/upload_max_filesize = 512M/' /etc/php/$PHP_VER/fpm/php.ini
    sed -i 's/post_max_size = .*/post_max_size = 512M/' /etc/php/$PHP_VER/fpm/php.ini

    systemctl restart php$PHP_VER-fpm
    systemctl restart mariadb
    success "System Tuned for $PROFILE usage."
}

init_system() {
    # OS Compatibility Check
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" != "debian" && "$ID" != "ubuntu" ]]; then
             warn "Detected OS: $ID. This script is optimized for Debian/Ubuntu."
             read -p "Continue anyway? (y/n): " confirm
             if [[ "$confirm" != "y" ]]; then exit 1; fi
        fi
    else
        warn "Cannot detect OS. Proceed with caution."
    fi

    if [ -f "/root/.server_installed" ]; then
        return
    fi
    msg "Initializing System Infrastructure..."
    
    # 1. Basics
    if ! command -v jq &> /dev/null; then
        msg "Installing Basic Dependencies..."
        apt-get update -q && apt-get install -y curl wget git unzip gnupg2 apt-transport-https ca-certificates lsb-release ufw sudo htop apache2-utils fail2ban jq bc >/dev/null
    fi
    
    # 2. Performance Tuning (Swap & Sysctl)
    if [ ! -f /swapfile ]; then
        msg "Creating Swap File..."
        fallocate -l 2G /swapfile
        chmod 600 /swapfile
        mkswap /swapfile
        swapon /swapfile
        echo '/swapfile none swap sw 0 0' >> /etc/fstab
    fi
    
    # TCP BBR (Speed Boost)
    if ! grep -q "tcp_bbr" /etc/sysctl.conf; then
        echo "net.core.default_qdisc=fq" >> /etc/sysctl.conf
        echo "net.ipv4.tcp_congestion_control=bbr" >> /etc/sysctl.conf
        sysctl -p >/dev/null
    fi
    
    # DNS Optimization (Google + Cloudflare)
    if [ -f /etc/systemd/resolved.conf ]; then
        msg "Configuring Optimized DNS..."
        sed -i 's/#DNS=/DNS=8.8.8.8 8.8.4.4 1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf
        sed -i 's/#FallbackDNS=/FallbackDNS=1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf
        systemctl restart systemd-resolved
    fi

    # 3. Docker Engine
    if ! command -v docker &> /dev/null; then
        curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
        apt-get update -q && apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin >/dev/null
    fi
    
    # Docker Network
    if ! docker network inspect $DOCKER_NET >/dev/null 2>&1; then
        docker network create $DOCKER_NET
    fi

    # 4. Host Stack (Nginx/PHP/MariaDB)
    apt-get install -y nginx mariadb-server certbot python3-certbot-nginx php-fpm php-mysql php-curl php-gd php-mbstring php-xml php-zip php-intl php-bcmath >/dev/null
    
    tune_system
    
    # 7. Security (UFW)
    ufw allow ssh >/dev/null
    ufw allow http >/dev/null
    ufw allow https >/dev/null
    SUBNET=$(docker network inspect $DOCKER_NET | jq -r '.[0].IPAM.Config[0].Subnet')
    ufw allow from $SUBNET to any port 3306 >/dev/null
    echo "y" | ufw enable >/dev/null

    # 8. Auth Init
    if [ ! -f $AUTH_FILE ]; then touch $AUTH_FILE && chmod 600 $AUTH_FILE; fi
    init_db_password
    

    
    setup_autoupdate
    
    # Save Install Dir for Auto-Update
    echo "INSTALL_DIR=$(pwd)" > /etc/cyl_manager.conf
    
    touch /root/.server_installed
    success "System Initialized"
}

init_db_password() {
    if grep -q "mysql_root_password" $AUTH_FILE; then
        DB_ROOT_PASS=$(grep "mysql_root_password" $AUTH_FILE | cut -d= -f2)
    else
        if [ -f /root/.mariadb_auth ]; then
             DB_ROOT_PASS=$(cat /root/.mariadb_auth | grep mysql_root_password | cut -d= -f2)
        else
             DB_ROOT_PASS=$(openssl rand -base64 12)
             echo "mysql_root_password=$DB_ROOT_PASS" >> $AUTH_FILE
             mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED BY '$DB_ROOT_PASS'; FLUSH PRIVILEGES;"
        fi
    fi
    export DB_ROOT_PASS
}

ensure_db() {
    mysql -u root --password="$DB_ROOT_PASS" -e "CREATE DATABASE IF NOT EXISTS $1; CREATE USER IF NOT EXISTS '$2'@'%' IDENTIFIED BY '$3'; GRANT ALL PRIVILEGES ON $1.* TO '$2'@'%'; FLUSH PRIVILEGES;"
}

# ------------------------------------------------------------------------------
# 4. SERVICE MODULES
# ------------------------------------------------------------------------------

update_nginx() {
    local sub=$1
    local port=$2
    local type=$3 # proxy or php
    local root=$4
    
    # Check if we need to remove default
    rm -f /etc/nginx/sites-enabled/default

    cat <<EOF > /etc/nginx/sites-available/$sub
server {
    listen 80;
    server_name $sub;
    client_max_body_size 10G;
EOF
    if [ "$type" == "proxy" ]; then
        cat <<EOF >> /etc/nginx/sites-available/$sub
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
        cat <<EOF >> /etc/nginx/sites-available/$sub
    root $root;
    index index.php index.html;
EOF
        if [ "$type" == "dashboard" ]; then
            echo '    location / { try_files $uri $uri/ =404; auth_basic "Restricted"; auth_basic_user_file /etc/nginx/.htpasswd; }' >> /etc/nginx/sites-available/$sub
        else
             echo '    location / { try_files $uri $uri/ =404; }' >> /etc/nginx/sites-available/$sub
        fi
        echo "    location ~ \.php$ { include snippets/fastcgi-php.conf; fastcgi_pass unix:$PHP_SOCK; }" >> /etc/nginx/sites-available/$sub
        echo "}" >> /etc/nginx/sites-available/$sub
    fi
    ln -sf /etc/nginx/sites-available/$sub /etc/nginx/sites-enabled/
}

manage_netdata() {
    if [ "$1" == "install" ]; then
        msg "Installing Netdata..."
        docker run -d --name=netdata --pid=host --network=$DOCKER_NET -p 127.0.0.1:19999:19999 \
          -v netdataconfig:/etc/netdata -v netdatalib:/var/lib/netdata -v netdatacache:/var/cache/netdata \
          -v /etc/passwd:/host/etc/passwd:ro -v /etc/group:/host/etc/group:ro -v /proc:/host/proc:ro \
          -v /sys:/host/sys:ro -v /etc/os-release:/host/etc/os-release:ro -v /var/run/docker.sock:/var/run/docker.sock \
          --restart always --cap-add SYS_PTRACE --security-opt apparmor=unconfined netdata/netdata
        update_nginx "netdata.$DOMAIN" "19999" "proxy"
        success "Netdata Installed"
    elif [ "$1" == "remove" ]; then
        docker stop netdata && docker rm netdata
        rm -f /etc/nginx/sites-enabled/netdata.$DOMAIN
        success "Netdata Removed"
    fi
}

manage_portainer() {
    if [ "$1" == "install" ]; then
        msg "Installing Portainer..."
        docker run -d -p 127.0.0.1:9000:9000 --name=portainer --network $DOCKER_NET --restart=always \
        -v /var/run/docker.sock:/var/run/docker.sock -v portainer_data:/data portainer/portainer-ce
        update_nginx "portainer.$DOMAIN" "9000" "proxy"
        success "Portainer Installed"
    elif [ "$1" == "remove" ]; then
        docker stop portainer && docker rm portainer
        rm -f /etc/nginx/sites-enabled/portainer.$DOMAIN
        success "Portainer Removed"
    fi
}

manage_gitea() {
    if [ "$1" == "install" ]; then
        msg "Installing Gitea..."
        mkdir -p /opt/gitea
        PASS=$(openssl rand -base64 12)
        ensure_db "gitea" "gitea" "$PASS"
        HOST_IP=$(hostname -I | awk '{print $1}')
        cat <<EOF > /opt/gitea/docker-compose.yml
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
      - GITEA__database__HOST=$HOST_IP:3306
      - GITEA__database__NAME=gitea
      - GITEA__database__USER=gitea
      - GITEA__database__PASSWD=$PASS
      - GITEA__server__SSH_DOMAIN=git.$DOMAIN
      - GITEA__server__SSH_PORT=2222
      - GITEA__server__ROOT_URL=https://git.$DOMAIN/
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
        cd /opt/gitea && docker compose up -d
        update_nginx "git.$DOMAIN" "3000" "proxy"
        success "Gitea Installed"
    elif [ "$1" == "remove" ]; then
        cd /opt/gitea && docker compose down
        rm -f /etc/nginx/sites-enabled/git.$DOMAIN
        success "Gitea Removed"
    fi
}

manage_nextcloud() {
    if [ "$1" == "install" ]; then
        msg "Installing Nextcloud..."
        mkdir -p /opt/nextcloud
        PASS=$(openssl rand -base64 12)
        ensure_db "nextcloud" "nextcloud" "$PASS"
        HOST_IP=$(hostname -I | awk '{print $1}')
        cat <<EOF > /opt/nextcloud/docker-compose.yml
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
      - MYSQL_PASSWORD=$PASS
      - MYSQL_DATABASE=nextcloud
      - MYSQL_USER=nextcloud
      - MYSQL_HOST=$HOST_IP
      - NEXTCLOUD_TRUSTED_DOMAINS=cloud.$DOMAIN
networks:
  $DOCKER_NET:
    external: true
EOF
        cd /opt/nextcloud && docker compose up -d
        msg "Waiting for Nextcloud to initialize..."
        until docker exec -u www-data nextcloud_app php occ status >/dev/null 2>&1; do
            sleep 2
        done
        docker exec -u www-data nextcloud_app php occ config:system:set trusted_proxies 0 --value="127.0.0.1" >/dev/null 2>&1
        docker exec -u www-data nextcloud_app php occ config:system:set overwriteprotocol --value="https" >/dev/null 2>&1
        update_nginx "cloud.$DOMAIN" "8080" "proxy"
        success "Nextcloud Installed"
    elif [ "$1" == "remove" ]; then
        cd /opt/nextcloud && docker compose down
        rm -f /etc/nginx/sites-enabled/cloud.$DOMAIN
        success "Nextcloud Removed"
    fi
}

manage_mail() {
    if [ "$1" == "install" ]; then
        msg "Installing Mail Server..."
        mkdir -p /opt/mail
        
        # RESOURCE CHECK FOR CLAMAV
        PROFILE=$(cat /tmp/server_profile)
        if [ "$PROFILE" == "LOW" ]; then
            CLAMAV_STATE=0
            msg "Low RAM detected: Disabling ClamAV Antivirus to save ~1.5GB RAM."
        else
            CLAMAV_STATE=1
            msg "High RAM detected: Enabling ClamAV Antivirus."
        fi
        
        cat <<EOF > /opt/mail/docker-compose.yml
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
      - ENABLE_CLAMAV=$CLAMAV_STATE
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
EOF
        cd /opt/mail && docker compose up -d
        ufw allow 25,587,465,143,993/tcp >/dev/null
        update_nginx "mail.$DOMAIN" "8081" "proxy"
        
        msg "Waiting for Mailserver..."
        until docker exec mailserver setup email list >/dev/null 2>&1; do
            sleep 2
        done
        
        if ! docker exec mailserver setup email list | grep -q "postmaster"; then
            PASS=$(openssl rand -base64 12)
            docker exec mailserver setup email add postmaster@$DOMAIN "$PASS"
            docker exec mailserver setup config dkim
            echo "postmaster_pass=$PASS" >> $AUTH_FILE
        fi
        success "Mail Server Installed"
    elif [ "$1" == "remove" ]; then
        cd /opt/mail && docker compose down
        rm -f /etc/nginx/sites-enabled/mail.$DOMAIN
        ufw delete allow 25/tcp >/dev/null
        success "Mail Server Removed"
    fi
}

manage_yourls() {
    if [ "$1" == "install" ]; then
        msg "Installing YOURLS..."
        mkdir -p /var/www/yourls
        PASS=$(openssl rand -base64 12)
        ensure_db "yourls" "yourls" "$PASS"
        cd /var/www/yourls
        if [ ! -f "yourls-loader.php" ]; then
            wget -q https://github.com/YOURLS/YOURLS/archive/refs/heads/master.zip
            unzip -q -o master.zip && mv YOURLS-master/* . && rm -rf YOURLS-master master.zip
            cp user/config-sample.php user/config.php
            sed -i "s/yourls_db_user = 'root'/yourls_db_user = 'yourls'/" user/config.php
            sed -i "s/yourls_db_pass = ''/yourls_db_pass = '$PASS'/" user/config.php
            sed -i "s/yourls_db_name = 'yourls'/yourls_db_name = 'yourls'/" user/config.php
            sed -i "s|define( 'YOURLS_SITE', 'http://your-own-domain-here.com' );|define( 'YOURLS_SITE', 'https://x.$DOMAIN' );|" user/config.php
            sed -i "s/define( 'YOURLS_COOKIEKEY', 'modify this text' );/define( 'YOURLS_COOKIEKEY', '$(openssl rand -hex 16)' );/" user/config.php
        fi
        chown -R www-data:www-data /var/www/yourls
        update_nginx "x.$DOMAIN" "0" "php" "/var/www/yourls"
        success "YOURLS Installed"
    elif [ "$1" == "remove" ]; then
        rm -rf /var/www/yourls
        rm -f /etc/nginx/sites-enabled/x.$DOMAIN
        success "YOURLS Removed"
    fi
}

manage_ftp() {
    if [ "$1" == "install" ]; then
        msg "Installing FTP..."
        apt-get install -y vsftpd >/dev/null
        cp /etc/vsftpd.conf /etc/vsftpd.conf.bak 2>/dev/null
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
pasv_address=$(curl -s https://api.ipify.org)
EOF
        ufw allow 20,21,990/tcp >/dev/null
        ufw allow 40000:50000/tcp >/dev/null
        systemctl restart vsftpd
        
        # Create user if not exists
        if ! id "cyluser" &>/dev/null; then
            FTP_PASS=$(openssl rand -base64 12)
            useradd -m -d /var/www -s /bin/bash cyluser
            echo "cyluser:$FTP_PASS" | chpasswd
            usermod -a -G www-data cyluser
            chown -R cyluser:www-data /var/www
            echo "ftp_user=cyluser" >> $AUTH_FILE
            echo "ftp_pass=$FTP_PASS" >> $AUTH_FILE
        fi
        success "FTP Installed"
    elif [ "$1" == "remove" ]; then
        apt-get remove -y vsftpd >/dev/null
        success "FTP Removed"
    fi
}

manage_vaultwarden() {
    if [ "$1" == "install" ]; then
        msg "Installing Vaultwarden..."
        mkdir -p /opt/vaultwarden
        cat <<EOF > /opt/vaultwarden/docker-compose.yml
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
        cd /opt/vaultwarden && docker compose up -d
        update_nginx "pass.$DOMAIN" "8082" "proxy"
        success "Vaultwarden Installed"
    elif [ "$1" == "remove" ]; then
        cd /opt/vaultwarden && docker compose down
        rm -f /etc/nginx/sites-enabled/pass.$DOMAIN
        success "Vaultwarden Removed"
    fi
}

manage_uptimekuma() {
    if [ "$1" == "install" ]; then
        msg "Installing Uptime Kuma..."
        mkdir -p /opt/uptimekuma
        cat <<EOF > /opt/uptimekuma/docker-compose.yml
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
        cd /opt/uptimekuma && docker compose up -d
        update_nginx "status.$DOMAIN" "3001" "proxy"
        success "Uptime Kuma Installed"
    elif [ "$1" == "remove" ]; then
        cd /opt/uptimekuma && docker compose down
        rm -f /etc/nginx/sites-enabled/status.$DOMAIN
        success "Uptime Kuma Removed"
    fi
}

manage_filebrowser() {
    if [ "$1" == "install" ]; then
        msg "Installing File Browser..."
        mkdir -p /opt/filebrowser
        touch /opt/filebrowser/filebrowser.db

        cat <<EOF > /opt/filebrowser/docker-compose.yml
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
        # Default settings to avoid admin/admin
        echo '{"port": 80, "baseURL": "", "address": "", "log": "stdout", "database": "/database.db", "root": "/srv"}' > /opt/filebrowser/settings.json

        cd /opt/filebrowser && docker compose up -d

        # We need to set password, but filebrowser CLI is inside container
        # This is a bit tricky on first run.
        # For simplicity, we let user handle initial login or use default admin/admin and warn them?
        # Better: Print instructions.

        update_nginx "files.$DOMAIN" "8083" "proxy"
        success "File Browser Installed. Default login: admin/admin (Change immediately!)"
    elif [ "$1" == "remove" ]; then
        cd /opt/filebrowser && docker compose down
        rm -f /etc/nginx/sites-enabled/files.$DOMAIN
        success "File Browser Removed"
    fi
}

manage_wireguard() {
    if [ "$1" == "install" ]; then
        msg "Installing WireGuard (WG-Easy)..."
        mkdir -p /opt/wireguard

        read -p "Enter WG Password (leave empty for auto): " WGPASS
        if [ -z "$WGPASS" ]; then
            WGPASS=$(openssl rand -base64 12)
            msg "Generated Password: $WGPASS"
            echo "wg_pass=$WGPASS" >> $AUTH_FILE
        fi

        HOST_IP=$(curl -s https://api.ipify.org)

        cat <<EOF > /opt/wireguard/docker-compose.yml
version: "3.8"
services:
  wg-easy:
    environment:
      - WG_HOST=$HOST_IP
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
        cd /opt/wireguard && docker compose up -d

        ufw allow 51820/udp >/dev/null
        update_nginx "vpn.$DOMAIN" "51821" "proxy"

        success "WireGuard Installed. UI at https://vpn.$DOMAIN"
    elif [ "$1" == "remove" ]; then
        cd /opt/wireguard && docker compose down
        rm -f /etc/nginx/sites-enabled/vpn.$DOMAIN
        ufw delete allow 51820/udp >/dev/null
        success "WireGuard Removed"
    fi
}

manage_backup() {
    msg "Starting Backup..."
    mkdir -p $BACKUP_DIR
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    
    # DB Dump
    msg "Backing up Databases..."
    mysqldump -u root --password="$DB_ROOT_PASS" --all-databases > $BACKUP_DIR/db_$TIMESTAMP.sql
    
    # Files
    msg "Backing up Files..."
    # Exclude backup dir itself if it's inside /opt or /var (it is in /var/backups, so safe from /var/www)
    tar -czf $BACKUP_DIR/files_$TIMESTAMP.tar.gz /opt /var/www /etc/nginx/sites-available /root/.auth_details 2>/dev/null
    
    # Retention (keep last 5)
    ls -tp $BACKUP_DIR/db_*.sql | grep -v '/$' | tail -n +6 | xargs -I {} rm -- {} 2>/dev/null || true
    ls -tp $BACKUP_DIR/files_*.tar.gz | grep -v '/$' | tail -n +6 | xargs -I {} rm -- {} 2>/dev/null || true
    
    success "Backup Complete: $TIMESTAMP"
}

system_update() {
    msg "Updating System Packages..."
    apt-get update && apt-get upgrade -y >/dev/null
    
    msg "Updating Docker Images..."
    docker images --format "{{.Repository}}:{{.Tag}}" | grep -v "<none>" | xargs -L1 docker pull 2>/dev/null || true
    docker image prune -f >/dev/null
    
    success "System Updated"
}

show_dns_records() {
    IP=$(curl -s https://api.ipify.org)
    msg "REQUIRED DNS RECORDS FOR $DOMAIN"
    echo -e "${YELLOW}-----------------------------------------------------${NC}"
    echo -e "TYPE  | HOST                 | VALUE"
    echo -e "${YELLOW}-----------------------------------------------------${NC}"
    echo -e "A     | @                    | $IP"
    echo -e "CNAME | www                  | $DOMAIN"
    echo -e "CNAME | admin                | $DOMAIN"
    
    [ -d "/opt/gitea" ] && echo -e "CNAME | git                  | $DOMAIN"
    [ -d "/opt/nextcloud" ] && echo -e "CNAME | cloud                | $DOMAIN"
    [ -d "/opt/mail" ] && echo -e "CNAME | mail                 | $DOMAIN"
    [ -d "/var/www/yourls" ] && echo -e "CNAME | x                    | $DOMAIN"
    [ -d "/opt/vaultwarden" ] && echo -e "CNAME | pass                 | $DOMAIN"
    [ -d "/opt/uptimekuma" ] && echo -e "CNAME | status               | $DOMAIN"
    docker ps | grep -q portainer && echo -e "CNAME | portainer            | $DOMAIN"
    docker ps | grep -q netdata && echo -e "CNAME | netdata              | $DOMAIN"
    
    echo -e "${YELLOW}-----------------------------------------------------${NC}"
    echo -e "MX    | @                    | mail.$DOMAIN (Priority 10)"
    echo -e "TXT   | @                    | v=spf1 mx ~all"
    echo -e "${YELLOW}-----------------------------------------------------${NC}"
    read -p "Press Enter to continue..." dummy >&3
}

manage_ssh() {
    msg "Hardening SSH Security..."
    
    # 1. Check for Keys
    if [ ! -f /root/.ssh/authorized_keys ] || [ ! -s /root/.ssh/authorized_keys ]; then
        warn "NO SSH KEYS FOUND!"
        echo "You must add your public key to /root/.ssh/authorized_keys before disabling passwords."
        echo "Otherwise you will be LOCKED OUT."
        read -p "Abort? (y/n): " confirm >&3
        if [[ "$confirm" != "n" ]]; then return; fi
    fi
    
    # 2. Backup
    cp /etc/ssh/sshd_config /etc/ssh/sshd_config.bak
    
    # 3. Configure
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
    sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
    sed -i 's/PermitRootLogin yes/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config

    # Change SSH Port
    read -p "Change SSH Port? (current: 22, default: 22) [y/N]: " change_port
    if [[ "$change_port" =~ ^[Yy]$ ]]; then
        read -p "Enter new SSH Port (e.g., 2222): " SSH_PORT
        if [[ "$SSH_PORT" =~ ^[0-9]+$ ]]; then
            sed -i "s/#Port 22/Port $SSH_PORT/" /etc/ssh/sshd_config
            sed -i "s/Port 22/Port $SSH_PORT/" /etc/ssh/sshd_config
            ufw allow $SSH_PORT/tcp >/dev/null
            if [ "$SSH_PORT" != "22" ]; then
                ufw delete allow 22/tcp >/dev/null
            fi
            msg "SSH Port changed to $SSH_PORT. Don't forget to update your client!"
        else
            warn "Invalid port. Skipping port change."
        fi
    fi
    
    # 4. Restart
    systemctl restart sshd
    success "SSH Hardened & Configured"
}

setup_autoupdate() {
    msg "Configuring Auto-Update..."
    
    # Copy script to /usr/local/bin
    cp $(dirname "$0")/auto_update.sh /usr/local/bin/server_autoupdate.sh
    chmod +x /usr/local/bin/server_autoupdate.sh
    
    # Add to crontab (run daily at 4:00 AM)
    CRON_CMD="0 4 * * * /usr/local/bin/server_autoupdate.sh"
    
    # Check if already exists
    (crontab -l 2>/dev/null | grep -v "server_autoupdate.sh"; echo "$CRON_CMD") | crontab -
    
    success "Auto-Update Scheduled (Daily @ 04:00)"
    msg "Logs will be available at: /var/log/server_autoupdate.log"
}

# ------------------------------------------------------------------------------
# 5. SYNC & MENU
# ------------------------------------------------------------------------------

sync_infrastructure() {
    msg "Syncing Dashboard & SSL..."
    mkdir -p /var/www/dashboard
    if ! grep -q "dashboard_pass" $AUTH_FILE; then
        PASS=$(openssl rand -base64 12)
        echo "dashboard_user=admin" >> $AUTH_FILE
        echo "dashboard_pass=$PASS" >> $AUTH_FILE
        htpasswd -bc /etc/nginx/.htpasswd admin $PASS
    fi
    
    LINKS="<a href='/adminer.php' class='card' target='_blank'>DB Admin</a>"
    [ -d "/opt/gitea" ] && LINKS+="<a href='https://git.$DOMAIN' class='card'>Gitea</a>"
    [ -d "/opt/nextcloud" ] && LINKS+="<a href='https://cloud.$DOMAIN' class='card'>Nextcloud</a>"
    [ -d "/opt/mail" ] && LINKS+="<a href='https://mail.$DOMAIN' class='card'>Webmail</a>"
    [ -d "/var/www/yourls" ] && LINKS+="<a href='https://x.$DOMAIN' class='card'>Shortener</a>"
    [ -d "/opt/vaultwarden" ] && LINKS+="<a href='https://pass.$DOMAIN' class='card'>Vaultwarden</a>"
    [ -d "/opt/uptimekuma" ] && LINKS+="<a href='https://status.$DOMAIN' class='card'>Status</a>"
    [ -d "/opt/wireguard" ] && LINKS+="<a href='https://vpn.$DOMAIN' class='card'>VPN</a>"
    [ -d "/opt/filebrowser" ] && LINKS+="<a href='https://files.$DOMAIN' class='card'>Files</a>"
    docker ps | grep -q portainer && LINKS+="<a href='https://portainer.$DOMAIN' class='card'>Portainer</a>"
    docker ps | grep -q netdata && LINKS+="<a href='https://netdata.$DOMAIN' class='card'>Monitoring</a>"

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
        .status { font-size: 0.8rem; margin-top: 10px; opacity: 0.7; }
    </style>
</head>
<body>
    <h1>Server Dashboard</h1>
    <p>System Profile: $(cat /tmp/server_profile) | <a href='/adminer.php' style='color:var(--primary)'>DB Admin</a></p>
    <div class='grid'>$LINKS</div>
</body>
</html>
EOF
    [ ! -f /var/www/dashboard/adminer.php ] && wget -q https://github.com/vrana/adminer/releases/download/v$ADMINER_VER/adminer-$ADMINER_VER.php -O /var/www/dashboard/adminer.php
    chown -R www-data:www-data /var/www/dashboard
    
    # Admin Dashboard on admin.domain
    update_nginx "admin.$DOMAIN" "0" "dashboard" "/var/www/dashboard"
    
    # Landing Page on root domain
    mkdir -p /var/www/landing
    cat <<EOF > /var/www/landing/index.html
<!DOCTYPE html><html><head><title>Welcome to $DOMAIN</title><style>body{background:#000;color:#fff;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;font-family:sans-serif}h1{font-weight:300;letter-spacing:5px}</style></head><body><h1>$DOMAIN</h1></body></html>
EOF
    chown -R www-data:www-data /var/www/landing
    update_nginx "$DOMAIN" "0" "php" "/var/www/landing"

    nginx -t && systemctl reload nginx
    
    # Certbot
    DOMAINS="-d $DOMAIN -d admin.$DOMAIN"
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
    
    echo "Updating SSL for: $DOMAINS"
    certbot --nginx $DOMAINS --non-interactive --agree-tos -m $EMAIL --redirect --expand
    
    success "Infrastructure Synced."
}

show_menu() {
    clear >&3
    PROFILE=$(cat /tmp/server_profile 2>/dev/null || echo "DETECTING...")
    
    # Health Stats
    LOAD=$(uptime | awk -F'load average:' '{ print $2 }' | cut -d, -f1 | xargs)
    DISK=$(df -h / | awk 'NR==2 {print $5}')
    RAM=$(free -m | awk '/Mem:/ {printf "%.0f%%", $3/$2*100}')
    
    echo -e "${PURPLE}=== MANAGER V6.0 - $DOMAIN [$PROFILE] ===${NC}" >&3
    echo -e "${BLUE}Health: Load: $LOAD | Disk: $DISK | RAM: $RAM${NC}" >&3
    echo -e "-----------------------------" >&3
    
    status_gitea=$( [ -d "/opt/gitea" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_next=$( [ -d "/opt/nextcloud" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_port=$( docker ps | grep -q portainer && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_netd=$( docker ps | grep -q netdata && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_mail=$( [ -d "/opt/mail" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_url=$( [ -d "/var/www/yourls" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_ftp=$( command -v vsftpd &>/dev/null && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_vault=$( [ -d "/opt/vaultwarden" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_kuma=$( [ -d "/opt/uptimekuma" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_wg=$( [ -d "/opt/wireguard" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )
    status_files=$( [ -d "/opt/filebrowser" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}" )

    echo -e "1. Gitea       [$status_gitea]" >&3
    echo -e "2. Nextcloud   [$status_next]" >&3
    echo -e "3. Portainer   [$status_port]" >&3
    echo -e "4. Netdata     [$status_netd]" >&3
    echo -e "5. Mail Server [$status_mail]" >&3
    echo -e "6. YOURLS      [$status_url]" >&3
    echo -e "7. FTP Server  [$status_ftp]" >&3
    echo -e "8. Vaultwarden [$status_vault]" >&3
    echo -e "9. Uptime Kuma [$status_kuma]" >&3
    echo -e "17. WireGuard  [$status_wg]" >&3
    echo -e "18. FileBrowse [$status_files]" >&3
    echo -e "-----------------------------" >&3
    echo -e "10. SYSTEM UPDATE" >&3
    echo -e "11. BACKUP NOW" >&3
    echo -e "12. SYNC ALL (SSL & Dashboard)" >&3
    echo -e "13. RE-TUNE SYSTEM (Apply $PROFILE settings)" >&3
    echo -e "14. FORCE RE-INIT SYSTEM" >&3

    echo -e "15. SHOW DNS RECORDS" >&3
    echo -e "16. HARDEN SSH (Keys Only)" >&3
    echo -e "0. Exit" >&3
    echo -e "" >&3
}

# --- EXECUTION ---

detect_profile
init_system

while true; do
    show_menu
    read -p "Select > " choice >&3
    
    case $choice in
        1) [ -d "/opt/gitea" ] && manage_gitea "remove" || manage_gitea "install" ;;
        2) [ -d "/opt/nextcloud" ] && manage_nextcloud "remove" || manage_nextcloud "install" ;;
        3) docker ps | grep -q portainer && manage_portainer "remove" || manage_portainer "install" ;;
        4) docker ps | grep -q netdata && manage_netdata "remove" || manage_netdata "install" ;;
        5) [ -d "/opt/mail" ] && manage_mail "remove" || manage_mail "install" ;;
        6) [ -d "/var/www/yourls" ] && manage_yourls "remove" || manage_yourls "install" ;;
        7) command -v vsftpd &>/dev/null && manage_ftp "remove" || manage_ftp "install" ;;
        8) [ -d "/opt/vaultwarden" ] && manage_vaultwarden "remove" || manage_vaultwarden "install" ;;
        9) [ -d "/opt/uptimekuma" ] && manage_uptimekuma "remove" || manage_uptimekuma "install" ;;
        10) system_update ;;
        11) manage_backup ;;
        12) sync_infrastructure ;;
        13) tune_system ;;
        14) rm -f /root/.server_installed && init_system ;;

        15) show_dns_records ;;
        16) manage_ssh ;;
        17) [ -d "/opt/wireguard" ] && manage_wireguard "remove" || manage_wireguard "install" ;;
        18) [ -d "/opt/filebrowser" ] && manage_filebrowser "remove" || manage_filebrowser "install" ;;
        0) echo "Bye!" >&3; exit 0 ;;
        *) echo "Invalid option" >&3 ;;
    esac
    
    if [[ "$choice" =~ [1-9] ]]; then
        read -p "Apply changes now? (y/n): " confirm >&3
        if [[ "$confirm" == "y" ]]; then sync_infrastructure; fi
    fi
    
    if [[ "$choice" -ge 10 ]]; then
        read -p "Press Enter to continue..." dummy >&3
    fi
done
