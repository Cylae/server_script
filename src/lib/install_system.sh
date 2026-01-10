#!/bin/bash
set -euo pipefail

# ==============================================================================
# INSTALL MODULE
# Initial system setup and dependencies
# ==============================================================================

init_system() {
    msg "Initializing System Infrastructure..."

    export DEBIAN_FRONTEND=noninteractive

    # 1. Basics
    if ! command -v jq &> /dev/null; then
        msg "Installing Basic Dependencies..."
        apt-get update -q && apt-get install -y curl wget git unzip gnupg2 apt-transport-https ca-certificates lsb-release ufw sudo htop apache2-utils fail2ban jq bc iproute2 ncurses-bin >/dev/null
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

    if ! grep -q "CYLAE OPTIMIZED NETWORK STACK" /etc/sysctl.conf; then
        cat <<EOF >> /etc/sysctl.conf
# ------------------------------------------------------------------------------
# CYLAE OPTIMIZED NETWORK STACK
# ------------------------------------------------------------------------------
# BBR Congestion Control
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr

# TCP Stack Tuning
net.ipv4.tcp_fastopen=3
net.ipv4.tcp_slow_start_after_idle=0
net.ipv4.tcp_tw_reuse=1
net.ipv4.tcp_max_syn_backlog=8192
net.core.somaxconn=8192
net.core.netdev_max_backlog=16384
fs.file-max=100000
# ------------------------------------------------------------------------------
EOF
        sysctl -p >/dev/null
    fi

    # 3. DNS Optimization
    if [ -f /etc/systemd/resolved.conf ]; then
        sed -i 's/^#\?DNS=.*/DNS=8.8.8.8 8.8.4.4 1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf
        sed -i 's/^#\?FallbackDNS=.*/FallbackDNS=1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf
        systemctl restart systemd-resolved || true
    fi

    # 4. Docker Engine (Official)
    init_docker

    # 5. Host Stack (Nginx/PHP/MariaDB/Certbot)
    msg "Installing Host Stack..."
    apt-get install -y nginx mariadb-server certbot python3-certbot-nginx php-fpm php-mysql php-curl php-gd php-mbstring php-xml php-zip php-intl php-bcmath >/dev/null

    detect_profile
    tune_system

    # 6. Firewall & Security (Phase 1)
    # We call security module functions here
    harden_system

    # Allow Docker subnet to access Host MariaDB
    SUBNET=$(docker network inspect $DOCKER_NET | jq -r '.[0].IPAM.Config[0].Subnet')
    ufw allow from "$SUBNET" to any port 3306 >/dev/null

    # 7. Auth Init
    if [ ! -f $AUTH_FILE ]; then touch $AUTH_FILE && chmod 600 $AUTH_FILE; fi
    init_db_password

    setup_autoupdate

    touch /root/.server_installed
    success "System Initialized"
}
