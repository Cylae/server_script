#!/bin/bash

# ==============================================================================
#  SERVER INSTALLATION SCRIPT / SCRIPT D'INSTALLATION SERVEUR
#  Domain: cyl.ae
#  Author: Jules
# ==============================================================================

# ------------------------------------------------------------------------------
# 1. VISUALS & VARIABLES / VISUELS ET VARIABLES
# ------------------------------------------------------------------------------

# Colors / Couleurs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default Language / Langue par défaut
LANG_CHOICE="EN"

# Domain / Domaine
DOMAIN="cyl.ae"

# ------------------------------------------------------------------------------
# 2. HELPER FUNCTIONS / FONCTIONS UTILES
# ------------------------------------------------------------------------------

# Print Header / Afficher l'en-tête
print_header() {
    clear
    echo -e "${CYAN}======================================================${NC}"
    echo -e "${CYAN}      ULTIMATE SERVER SETUP FOR ${DOMAIN}      ${NC}"
    echo -e "${CYAN}======================================================${NC}"
    echo ""
}

# Print Info / Afficher Info
info() {
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${BLUE}[INFO]${NC} $2"
    else
        echo -e "${BLUE}[INFO]${NC} $1"
    fi
}

# Print Success / Afficher Succès
success() {
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${GREEN}[OK]${NC} $2"
    else
        echo -e "${GREEN}[OK]${NC} $1"
    fi
}

# Print Warning / Afficher Avertissement
warn() {
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${YELLOW}[ATTENTION]${NC} $2"
    else
        echo -e "${YELLOW}[WARNING]${NC} $1"
    fi
}

# Print Error / Afficher Erreur
error() {
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${RED}[ERREUR]${NC} $2"
    else
        echo -e "${RED}[ERROR]${NC} $1"
    fi
    exit 1
}

# ------------------------------------------------------------------------------
# 3. INITIAL CHECKS / VÉRIFICATIONS INITIALES
# ------------------------------------------------------------------------------

# Check Root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root / Ce script doit être lancé en root${NC}"
   exit 1
fi

# Language Selection / Sélection de la langue
choose_language() {
    print_header
    echo -e "Please select your language / Veuillez choisir votre langue :"
    echo -e "1) English (Default)"
    echo -e "2) Français"
    read -p "Choice/Choix [1]: " lang_input

    if [[ "$lang_input" == "2" ]]; then
        LANG_CHOICE="FR"
        info "Language set to French." "Langue définie sur Français."
    else
        LANG_CHOICE="EN"
        info "Language set to English." "Langue définie sur Anglais."
    fi
}

# ------------------------------------------------------------------------------
# 4. SYSTEM SETUP / CONFIGURATION SYSTÈME
# ------------------------------------------------------------------------------

update_system() {
    info "Updating system repositories and packages..." "Mise à jour des dépôts et paquets..."
    apt-get update -y && apt-get upgrade -y
    apt-get install -y curl wget git unzip gnupg2 apt-transport-https ca-certificates lsb-release ufw sudo htop
    success "System updated." "Système mis à jour."
}

create_swap() {
    info "Checking Swap configuration..." "Vérification de la configuration Swap..."

    # Check if swap exists
    if grep -q "swap" /proc/swaps; then
        warn "Swap already exists. Skipping creation." "Le Swap existe déjà. Création ignorée."
    else
        info "Creating a 2GB Swap file for better performance on small VMs..." "Création d'un fichier Swap de 2Go pour les petites VM..."
        fallocate -l 2G /swapfile
        chmod 600 /swapfile
        mkswap /swapfile
        swapon /swapfile
        echo '/swapfile none swap sw 0 0' | tee -a /etc/fstab
        success "Swap file created." "Fichier Swap créé."
    fi
}

# ------------------------------------------------------------------------------
# 5. CORE STACK INSTALLATION / INSTALLATION DU NOYAU
# ------------------------------------------------------------------------------

install_docker() {
    info "Checking Docker installation..." "Vérification de l'installation Docker..."
    if command -v docker &> /dev/null; then
        warn "Docker is already installed." "Docker est déjà installé."
    else
        info "Installing Docker & Docker Compose..." "Installation de Docker et Docker Compose..."
        mkdir -p /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        echo \
          "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
          $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
        apt-get update -y
        apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin docker-compose
        success "Docker installed." "Docker installé."
    fi
}

install_nginx_php() {
    info "Installing Nginx and PHP..." "Installation de Nginx et PHP..."
    apt-get install -y nginx
    # Installing PHP 8.2 (default in Debian 12) or latest available
    apt-get install -y php-cli php-fpm php-mysql php-curl php-gd php-mbstring php-xml php-xmlrpc php-soap php-intl php-zip

    # Enable Nginx
    systemctl enable nginx
    systemctl start nginx
    success "Nginx and PHP installed." "Nginx et PHP installés."
}

get_php_socket() {
    # Detect PHP version
    PHP_VERSION=$(php -r "echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;")
    echo "unix:/run/php/php${PHP_VERSION}-fpm.sock"
}

install_mariadb() {
    info "Installing MariaDB Server..." "Installation du serveur MariaDB..."
    apt-get install -y mariadb-server
    systemctl enable mariadb
    systemctl start mariadb
    success "MariaDB installed." "MariaDB installé."
}

install_certbot() {
    info "Installing Certbot..." "Installation de Certbot..."
    apt-get install -y certbot python3-certbot-nginx
    success "Certbot installed." "Certbot installé."
}

# ------------------------------------------------------------------------------
# 6. SECURITY & DATABASE / SÉCURITÉ ET BASE DE DONNÉES
# ------------------------------------------------------------------------------

configure_security() {
    info "Configuring Firewall (UFW)..." "Configuration du pare-feu (UFW)..."
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow ssh
    ufw allow http
    ufw allow https
    # FTP
    ufw allow 20/tcp
    ufw allow 21/tcp
    ufw allow 990/tcp
    ufw allow 40000:50000/tcp
    # Mail
    ufw allow 25
    ufw allow 587
    ufw allow 465
    ufw allow 143
    ufw allow 993

    # Allow Docker containers to access Host MariaDB (3306)
    # Assuming standard Docker subnet 172.17.0.0/16 or similar. We allow from any 172.xx private range for simplicity
    ufw allow from 172.16.0.0/12 to any port 3306

    # Enable
    echo "y" | ufw enable
    success "Firewall configured." "Pare-feu configuré."
}

configure_mariadb() {
    info "Configuring MariaDB..." "Configuration de MariaDB..."

    # Generate a random root password
    DB_ROOT_PASS=$(openssl rand -base64 12)
    echo "mysql_root_password=$DB_ROOT_PASS" > /root/.mariadb_auth
    chmod 600 /root/.mariadb_auth

    # Secure installation programmatically
    mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED BY '$DB_ROOT_PASS';"
    mysql -e "FLUSH PRIVILEGES;"

    # Enable remote access (for Docker containers) but protected by UFW
    sed -i 's/^bind-address\s*=.*/bind-address            = 0.0.0.0/' /etc/mysql/mariadb.conf.d/50-server.cnf
    systemctl restart mariadb

    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${YELLOW}Mot de passe Root MariaDB généré : $DB_ROOT_PASS (sauvegardé dans /root/.mariadb_auth)${NC}"
    else
        echo -e "${YELLOW}MariaDB Root Password generated: $DB_ROOT_PASS (saved in /root/.mariadb_auth)${NC}"
    fi

    success "MariaDB secured." "MariaDB sécurisé."
}

create_app_db() {
    local dbname=$1
    local dbuser=$2
    local dbpass=$3

    mysql -u root --password="$DB_ROOT_PASS" -e "CREATE DATABASE IF NOT EXISTS $dbname;"
    mysql -u root --password="$DB_ROOT_PASS" -e "CREATE USER IF NOT EXISTS '$dbuser'@'%' IDENTIFIED BY '$dbpass';"
    mysql -u root --password="$DB_ROOT_PASS" -e "GRANT ALL PRIVILEGES ON $dbname.* TO '$dbuser'@'%';"
    mysql -u root --password="$DB_ROOT_PASS" -e "FLUSH PRIVILEGES;"
}

# ------------------------------------------------------------------------------
# 7. SERVICES PART 1: FTP, GIT, NEXTCLOUD
# ------------------------------------------------------------------------------

install_ftp() {
    info "Installing vsftpd (FTP/sFTP)..." "Installation de vsftpd (FTP/sFTP)..."
    apt-get install -y vsftpd

    # Backup original config
    mv /etc/vsftpd.conf /etc/vsftpd.conf.bak

    # Create new config (Enabling SSL/TLS)
    # We use snakeoil certs initially, but user should use valid certs ideally.
    cat <<EOF > /etc/vsftpd.conf
listen=NO
listen_ipv6=YES
anonymous_enable=NO
local_enable=YES
write_enable=YES
local_umask=022
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
force_local_data_ssl=YES
force_local_logins_ssl=YES
ssl_tlsv1=YES
ssl_sslv2=NO
ssl_sslv3=NO
require_ssl_reuse=NO
ssl_ciphers=HIGH
pasv_min_port=40000
pasv_max_port=50000
allow_writeable_chroot=YES
EOF

    # Create FTP User
    FTP_USER="cyluser"
    FTP_PASS=$(openssl rand -base64 12)
    echo "ftp_user=$FTP_USER" >> /root/.auth_details
    echo "ftp_pass=$FTP_PASS" >> /root/.auth_details

    # User home is /var/www to access all sub-services (yourls, dashboard)
    # shell must be valid for sFTP. We use /bin/bash but rely on chroot in vsftpd for FTP
    # For sFTP (SSH), we must rely on sshd_config.

    useradd -m -d /var/www -s /bin/bash $FTP_USER
    echo "$FTP_USER:$FTP_PASS" | chpasswd

    # Set group permissions so www-data (nginx) and ftp user can both work
    usermod -a -G www-data $FTP_USER
    chown -R $FTP_USER:www-data /var/www
    chmod -R 775 /var/www

    systemctl restart vsftpd

    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${YELLOW}Utilisateur FTP créé : $FTP_USER / $FTP_PASS${NC}"
        echo -e "${YELLOW}Note: Compatible sFTP via Port 22.${NC}"
    else
        echo -e "${YELLOW}FTP User created: $FTP_USER / $FTP_PASS${NC}"
        echo -e "${YELLOW}Note: sFTP compatible via Port 22.${NC}"
    fi
    success "FTP installed." "FTP installé."
}

install_gitea() {
    info "Installing Gitea (Git Server)..." "Installation de Gitea..."
    mkdir -p /opt/gitea

    GITEA_DB_PASS=$(openssl rand -base64 12)
    create_app_db "gitea" "gitea" "$GITEA_DB_PASS"

    # Get host IP for DB connection
    HOST_IP=$(hostname -I | awk '{print $1}')

    cat <<EOF > /opt/gitea/docker-compose.yml
version: "3"
services:
  server:
    image: gitea/gitea:latest
    container_name: gitea
    environment:
      - USER_UID=1000
      - USER_GID=1000
      - GITEA__database__DB_TYPE=mysql
      - GITEA__database__HOST=$HOST_IP:3306
      - GITEA__database__NAME=gitea
      - GITEA__database__USER=gitea
      - GITEA__database__PASSWD=$GITEA_DB_PASS
    restart: always
    volumes:
      - ./data:/data
      - /etc/timezone:/etc/timezone:ro
      - /etc/localtime:/etc/localtime:ro
    ports:
      - "127.0.0.1:3000:3000"
      - "2222:22"
EOF

    cd /opt/gitea
    docker compose up -d
    success "Gitea installed (Port 3000)." "Gitea installé (Port 3000)."
}

install_nextcloud() {
    info "Installing Nextcloud..." "Installation de Nextcloud..."
    mkdir -p /opt/nextcloud

    NC_DB_PASS=$(openssl rand -base64 12)
    create_app_db "nextcloud" "nextcloud" "$NC_DB_PASS"
    HOST_IP=$(hostname -I | awk '{print $1}')

    NC_ADMIN_PASS=$(openssl rand -base64 12)
    echo "nextcloud_admin_pass=$NC_ADMIN_PASS" >> /root/.auth_details

    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${YELLOW}Compte Admin Nextcloud (admin) : $NC_ADMIN_PASS${NC}"
    else
        echo -e "${YELLOW}Nextcloud Admin Account (admin): $NC_ADMIN_PASS${NC}"
    fi

    cat <<EOF > /opt/nextcloud/docker-compose.yml
version: '2'
services:
  app:
    image: nextcloud
    restart: always
    ports:
      - 127.0.0.1:8080:80
    volumes:
      - ./nextcloud:/var/www/html
    environment:
      - MYSQL_PASSWORD=$NC_DB_PASS
      - MYSQL_DATABASE=nextcloud
      - MYSQL_USER=nextcloud
      - MYSQL_HOST=$HOST_IP
      - NEXTCLOUD_TRUSTED_DOMAINS=cloud.$DOMAIN
      - NEXTCLOUD_ADMIN_USER=admin
      - NEXTCLOUD_ADMIN_PASSWORD=$NC_ADMIN_PASS
EOF

    cd /opt/nextcloud
    docker compose up -d

    # Configure Nextcloud for Reverse Proxy (Overwrite Protocol)

    info "Waiting for Nextcloud to initialize..." "Attente de l'initialisation de Nextcloud..."

    MAX_RETRIES=30
    COUNT=0
    while [ $COUNT -lt $MAX_RETRIES ]; do
        if docker exec -u www-data app php occ status > /dev/null 2>&1; then
            break
        fi
        sleep 10
        COUNT=$((COUNT+1))
    done

    # Attempt to add trusted proxies and overwriteprotocol
    docker exec -u www-data app php occ config:system:set trusted_proxies 0 --value="127.0.0.1"
    docker exec -u www-data app php occ config:system:set overwriteprotocol --value="https"

    success "Nextcloud installed (Port 8080)." "Nextcloud installé (Port 8080)."
}

# ------------------------------------------------------------------------------
# 8. SERVICES PART 2: SHORTENER, MAIL
# ------------------------------------------------------------------------------

install_yourls() {
    info "Installing YOURLS (URL Shortener)..." "Installation de YOURLS (Raccourcisseur d'URL)..."
    mkdir -p /var/www/yourls

    # Create DB
    YOURLS_DB_PASS=$(openssl rand -base64 12)
    create_app_db "yourls" "yourls" "$YOURLS_DB_PASS"

    # Download YOURLS
    cd /var/www/yourls
    wget -q https://github.com/YOURLS/YOURLS/archive/refs/heads/master.zip
    unzip -q master.zip
    mv YOURLS-master/* .
    rm -rf YOURLS-master master.zip

    # Config
    cp user/config-sample.php user/config.php

    # Configure DB in config.php
    # Pattern: define( 'YOURLS_DB_USER', 'root' );
    sed -i "s/define( 'YOURLS_DB_USER', 'root' );/define( 'YOURLS_DB_USER', 'yourls' );/" user/config.php
    sed -i "s/define( 'YOURLS_DB_PASS', '' );/define( 'YOURLS_DB_PASS', '$YOURLS_DB_PASS' );/" user/config.php
    sed -i "s/define( 'YOURLS_DB_NAME', 'yourls' );/define( 'YOURLS_DB_NAME', 'yourls' );/" user/config.php

    # Configure URL
    sed -i "s|define( 'YOURLS_SITE', 'http://your-own-domain-here.com' );|define( 'YOURLS_SITE', 'https://x.$DOMAIN' );|" user/config.php

    # Generate random cookie secret
    COOKIE_SECRET=$(openssl rand -hex 16)
    sed -i "s/define( 'YOURLS_COOKIEKEY', 'modify this text' );/define( 'YOURLS_COOKIEKEY', '$COOKIE_SECRET' );/" user/config.php

    # Permissions
    chown -R www-data:www-data /var/www/yourls

    success "YOURLS installed." "YOURLS installé."
}

install_mail() {
    info "Installing Mail Server (Docker Mailserver + Roundcube)..." "Installation du serveur Mail (Docker Mailserver + Roundcube)..."
    mkdir -p /opt/mail

    HOST_IP=$(hostname -I | awk '{print $1}')

    # We will use Roundcube in docker as well
    # Warning: Mail server configuration requires proper DNS (MX, SPF, DKIM)
    # This setup sets up the software side.

    cat <<EOF > /opt/mail/docker-compose.yml
version: '3'
services:
  mailserver:
    image: mailserver/docker-mailserver:latest
    hostname: mail
    domainname: cyl.ae
    container_name: mailserver
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
      - ENABLE_CLAMAV=0  # Disabled to save RAM on small VM / Désactivé pour économiser la RAM
      - ENABLE_FAIL2BAN=1
      - ENABLE_POSTGREY=1
      - SSL_TYPE=letsencrypt
    cap_add:
      - NET_ADMIN
    restart: always

  roundcube:
    image: roundcube/roundcubemail:latest
    container_name: roundcube
    restart: always
    volumes:
      - ./roundcube/db:/var/www/db
    ports:
      - "127.0.0.1:8081:80"
    environment:
      - ROUNDCUBEMAIL_DEFAULT_HOST=mailserver
      - ROUNDCUBEMAIL_SMTP_SERVER=mailserver
      - ROUNDCUBEMAIL_PLUGINS=archive,zipdownload
EOF

    cd /opt/mail
    docker compose up -d

    # Wait for container to start (Loop check)
    info "Waiting for mailserver to initialize (this may take a minute)..." "En attente de l'initialisation du serveur mail (cela peut prendre une minute)..."

    MAX_RETRIES=30
    COUNT=0
    while [ $COUNT -lt $MAX_RETRIES ]; do
        if docker exec mailserver ls /tmp/docker-mailserver > /dev/null 2>&1; then
            break
        fi
        sleep 5
        COUNT=$((COUNT+1))
    done

    # Create default postmaster account
    MAIL_PASS=$(openssl rand -base64 12)

    # Retry adding email if it fails initially
    sleep 5
    docker exec mailserver setup email add postmaster@$DOMAIN "$MAIL_PASS" || \
        (sleep 10 && docker exec mailserver setup email add postmaster@$DOMAIN "$MAIL_PASS")

    docker exec mailserver setup config dkim

    echo "postmaster_mail_pass=$MAIL_PASS" >> /root/.auth_details

    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${YELLOW}Compte Mail postmaster@$DOMAIN créé : $MAIL_PASS${NC}"
        warn "Note : Sur Google Cloud (GCP), le port 25 sortant est bloqué. Vous devrez configurer un relais SMTP (SendGrid, Mailgun...) pour envoyer des mails."
    else
        echo -e "${YELLOW}Mail Account postmaster@$DOMAIN created: $MAIL_PASS${NC}"
        warn "Note: On Google Cloud (GCP), outbound Port 25 is blocked. You must configure an SMTP Relay (SendGrid, Mailgun...) to send emails."
    fi

    success "Mail Server installed." "Serveur Mail installé."
}

# ------------------------------------------------------------------------------
# 9. UNIFIED DASHBOARD / TABLEAU DE BORD UNIFIÉ
# ------------------------------------------------------------------------------

create_dashboard() {
    info "Creating Unified Dashboard..." "Création du tableau de bord unifié..."
    mkdir -p /var/www/dashboard

    # Create Admin User for Dashboard
    DASH_USER="admin"
    DASH_PASS=$(openssl rand -base64 12)
    echo "dashboard_user=$DASH_USER" >> /root/.auth_details
    echo "dashboard_pass=$DASH_PASS" >> /root/.auth_details

    # Create .htpasswd
    htpasswd -bc /etc/nginx/.htpasswd $DASH_USER $DASH_PASS

    # Create index.php
    cat <<EOF > /var/www/dashboard/index.php
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Server Administration - cyl.ae</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background-color: #f4f4f9; color: #333; margin: 0; padding: 20px; }
        .container { max-width: 800px; margin: 0 auto; background: #fff; padding: 40px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; text-align: center; }
        .service-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-top: 30px; }
        .service-card { background: #ecf0f1; padding: 20px; border-radius: 8px; text-align: center; transition: transform 0.2s; text-decoration: none; color: inherit; display: block; }
        .service-card:hover { transform: translateY(-5px); background: #e0e6ed; }
        .service-icon { font-size: 40px; margin-bottom: 10px; }
        .service-name { font-weight: bold; font-size: 18px; }
        .footer { text-align: center; margin-top: 40px; font-size: 0.9em; color: #7f8c8d; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Admin Dashboard</h1>
        <p style="text-align: center;">Welcome to your server control center.</p>

        <div class="service-grid">
            <a href="https://git.cyl.ae" class="service-card" target="_blank">
                <div class="service-icon">🔨</div>
                <div class="service-name">Gitea (Git)</div>
            </a>
            <a href="https://cloud.cyl.ae" class="service-card" target="_blank">
                <div class="service-icon">☁️</div>
                <div class="service-name">Nextcloud</div>
            </a>
            <a href="https://mail.cyl.ae" class="service-card" target="_blank">
                <div class="service-icon">📧</div>
                <div class="service-name">Webmail</div>
            </a>
            <a href="https://x.cyl.ae/admin" class="service-card" target="_blank">
                <div class="service-icon">🔗</div>
                <div class="service-name">URL Shortener</div>
            </a>
             <a href="ftp://ftp.cyl.ae" class="service-card" target="_blank">
                <div class="service-icon">📂</div>
                <div class="service-name">FTP Access</div>
            </a>
             <a href="/adminer.php" class="service-card" target="_blank">
                <div class="service-icon">🗄️</div>
                <div class="service-name">DB Admin (Adminer)</div>
            </a>
        </div>

        <div class="info-section" style="margin-top: 40px; background: #fff; padding: 20px; border-radius: 8px;">
            <h3>Management Commands / Commandes de gestion</h3>
            <p><strong>Add Mail Account / Ajouter compte mail:</strong><br>
            <code>docker exec -ti mailserver setup email add user@cyl.ae password</code></p>

            <p><strong>Manage FTP Users / Gérer utilisateurs FTP:</strong><br>
            <code>useradd -m -d /var/www/newsite -s /bin/bash newuser</code><br>
            <code>passwd newuser</code></p>
        </div>

        <div class="footer">
            <p>Server managed by Script.</p>
        </div>
    </div>
</body>
</html>
EOF

    # Install Adminer for Database Management
    wget -q https://github.com/vrana/adminer/releases/download/v4.8.1/adminer-4.8.1.php -O /var/www/dashboard/adminer.php

    chown -R www-data:www-data /var/www/dashboard

    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "${YELLOW}Compte Admin Dashboard créé : $DASH_USER / $DASH_PASS${NC}"
        echo -e "${YELLOW}Adminer (Gestionnaire BDD) accessible sur le dashboard.${NC}"
    else
        echo -e "${YELLOW}Dashboard Admin Account created: $DASH_USER / $DASH_PASS${NC}"
        echo -e "${YELLOW}Adminer (DB Manager) accessible on dashboard.${NC}"
    fi
    success "Dashboard created." "Tableau de bord créé."
}

# ------------------------------------------------------------------------------
# 10. NGINX CONFIGURATION / CONFIGURATION NGINX
# ------------------------------------------------------------------------------

configure_nginx() {
    info "Configuring Nginx reverse proxies..." "Configuration des reverse proxies Nginx..."

    # 1. Dashboard (Main Domain or Admin Subdomain) -> cyl.ae
    # We use admin.cyl.ae or the root cyl.ae ? User said "Global admin page"
    # Let's put dashboard on cyl.ae (root)

    # Get Dynamic PHP Socket
    PHP_SOCKET=$(get_php_socket)

    cat <<EOF > /etc/nginx/sites-available/$DOMAIN
server {
    listen 80;
    server_name $DOMAIN;
    root /var/www/dashboard;
    index index.php index.html;

    location / {
        try_files \$uri \$uri/ =404;
        auth_basic "Restricted Access";
        auth_basic_user_file /etc/nginx/.htpasswd;
    }

    location ~ \.php$ {
        include snippets/fastcgi-php.conf;
        fastcgi_pass $PHP_SOCKET;
    }
}
EOF

    # 2. Gitea -> git.cyl.ae (Proxy to 3000)
    cat <<EOF > /etc/nginx/sites-available/git.$DOMAIN
server {
    listen 80;
    server_name git.$DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

    # 3. Nextcloud -> cloud.cyl.ae (Proxy to 8080)
    cat <<EOF > /etc/nginx/sites-available/cloud.$DOMAIN
server {
    listen 80;
    server_name cloud.$DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        client_max_body_size 10G;
    }
}
EOF

    # 4. URL Shortener -> x.cyl.ae (Native PHP)
    cat <<EOF > /etc/nginx/sites-available/x.$DOMAIN
server {
    listen 80;
    server_name x.$DOMAIN;
    root /var/www/yourls;
    index index.php index.html;

    location / {
        try_files \$uri \$uri/ /yourls-loader.php\$is_args\$args;
    }

    location ~ \.php$ {
        include snippets/fastcgi-php.conf;
        fastcgi_pass $PHP_SOCKET;
    }
}
EOF

    # 5. Mail Webmail -> mail.cyl.ae (Proxy to 8081)
    cat <<EOF > /etc/nginx/sites-available/mail.$DOMAIN
server {
    listen 80;
    server_name mail.$DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

    # Enable Sites
    ln -s /etc/nginx/sites-available/$DOMAIN /etc/nginx/sites-enabled/
    ln -s /etc/nginx/sites-available/git.$DOMAIN /etc/nginx/sites-enabled/
    ln -s /etc/nginx/sites-available/cloud.$DOMAIN /etc/nginx/sites-enabled/
    ln -s /etc/nginx/sites-available/x.$DOMAIN /etc/nginx/sites-enabled/
    ln -s /etc/nginx/sites-available/mail.$DOMAIN /etc/nginx/sites-enabled/

    rm /etc/nginx/sites-enabled/default

    nginx -t
    systemctl reload nginx
    success "Nginx Configured." "Nginx Configuré."
}

generate_ssl() {
    info "Generating SSL Certificates with Certbot..." "Génération des certificats SSL avec Certbot..."

    # We assume DNS is pointed. If not, this will fail.
    # We add --non-interactive and --agree-tos
    # We ask for email or use a dummy one? Prompting user is better.

    echo ""
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "IMPORTANT : Assurez-vous que les DNS (A records) pour :"
        echo -e "cyl.ae, git.cyl.ae, cloud.cyl.ae, x.cyl.ae, mail.cyl.ae"
        echo -e "pointent vers ce serveur AVANT de continuer."
        read -p "Est-ce le cas ? (o/n) " dns_confirm
    else
        echo -e "IMPORTANT: Ensure DNS (A records) for:"
        echo -e "cyl.ae, git.cyl.ae, cloud.cyl.ae, x.cyl.ae, mail.cyl.ae"
        echo -e "point to this server BEFORE continuing."
        read -p "Is this done? (y/n) " dns_confirm
    fi

    if [[ "$dns_confirm" == "y" || "$dns_confirm" == "o" ]]; then
        certbot --nginx -d $DOMAIN -d git.$DOMAIN -d cloud.$DOMAIN -d x.$DOMAIN -d mail.$DOMAIN --non-interactive --agree-tos -m admin@$DOMAIN --redirect
        success "SSL Certificates installed." "Certificats SSL installés."
    else
        warn "Skipping SSL generation. Run 'certbot --nginx' manually later." "Génération SSL ignorée. Lancez 'certbot --nginx' manuellement plus tard."
    fi
}

# ------------------------------------------------------------------------------
# MAIN EXECUTION / EXÉCUTION PRINCIPALE
# ------------------------------------------------------------------------------

# Call the functions if the script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    choose_language

    # Introduction
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "Bienvenue ! Je vais configurer votre serveur Debian pour ${CYAN}${DOMAIN}${NC}."
        echo -e "Installez-vous confortablement, je m'occupe de tout."
        echo -e "Services à installer : Nginx, PHP, MariaDB, Mail, Git, FTP, Nextcloud, URL Shortener."
    else
        echo -e "Welcome! I will configure your Debian server for ${CYAN}${DOMAIN}${NC}."
        echo -e "Sit back, I'll handle everything."
        echo -e "Services to install: Nginx, PHP, MariaDB, Mail, Git, FTP, Nextcloud, URL Shortener."
    fi

    echo ""
    read -p "Press Enter to start... / Appuyez sur Entrée pour commencer..."

    update_system
    create_swap

    install_docker
    install_nginx_php
    install_mariadb
    install_certbot

    configure_security
    configure_mariadb

    install_ftp
    install_gitea
    install_nextcloud

    install_yourls
    install_mail

    create_dashboard

    configure_nginx
    generate_ssl

    info "All services installed and dashboard created." "Tous les services sont installés et le tableau de bord est créé."

    echo ""
    echo -e "${CYAN}======================================================${NC}"
    if [ "$LANG_CHOICE" == "FR" ]; then
        echo -e "INSTALLATION TERMINÉE !"
        echo -e "Détails des accès sauvegardés dans /root/.auth_details"
        echo -e "Accédez à votre tableau de bord : https://cyl.ae"
    else
        echo -e "INSTALLATION COMPLETE!"
        echo -e "Access details saved in /root/.auth_details"
        echo -e "Access your dashboard: https://cyl.ae"
    fi
    echo -e "${CYAN}======================================================${NC}"
fi
