#!/bin/bash
set -euo pipefail

# ==============================================================================
# INSTALL MODULE
# Initial system setup and dependencies
# ==============================================================================

check_resources() {
    # CPU Check
    local CPU_CORES=$(nproc)
    if [ "$CPU_CORES" -lt 2 ]; then
        warn "Low CPU Core count detected ($CPU_CORES). Recommended: 2+"
    fi

    # Disk Check (Free space on /)
    local FREE_DISK=$(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//')

    if [ -z "$FREE_DISK" ]; then
        warn "Could not detect disk space. Skipping check."
    elif [ "$FREE_DISK" -lt 5 ]; then
        fatal "Insufficient Disk Space. Free: ${FREE_DISK}GB. Required: 5GB+"
    fi
}

calculate_swap_size() {
    local RAM_MB=$(free -m | awk '/Mem:/ {print $2}')

    # Get Free Disk Space in MB
    local FREE_DISK_MB=$(df -BM / | awk 'NR==2 {print $4}' | sed 's/M//')

    # 1. Calculate Ideal Swap based on RAM
    local ideal_swap
    if [ "$RAM_MB" -lt 2048 ]; then
        # < 2GB RAM: 2x RAM
        ideal_swap=$(( RAM_MB * 2 ))
    elif [ "$RAM_MB" -le 8192 ]; then
        # 2GB - 8GB RAM: 1x RAM
        ideal_swap="${RAM_MB}"
    else
        # > 8GB RAM: Cap at 4GB
        ideal_swap="4096"
    fi

    # 2. Safety Check: Don't consume more than 50% of available disk space
    local max_safe_swap=$(( FREE_DISK_MB / 2 ))

    # 3. Small Disk Logic: If Free Disk < 10GB, Cap Swap strictly
    if [ "$FREE_DISK_MB" -lt 10240 ]; then
        if [ "$ideal_swap" -gt 1024 ]; then
                ideal_swap=1024
        fi
    fi

    # Apply Safety Cap
    if [ "$ideal_swap" -gt "$max_safe_swap" ]; then
        echo "$max_safe_swap"
    else
        echo "$ideal_swap"
    fi
}

init_system() {
    check_resources
    msg "Initializing System Infrastructure..."

    export DEBIAN_FRONTEND=noninteractive

    # 1. Basics
    if ! command -v jq &> /dev/null || ! command -v pigz &> /dev/null; then
        msg "Installing Basic Dependencies..."
        apt-get update -q
        apt-get install -y curl wget git unzip gnupg2 apt-transport-https ca-certificates lsb-release ufw sudo htop apache2-utils fail2ban jq bc iproute2 ncurses-bin pigz unattended-upgrades >/dev/null

        # Configure Unattended Upgrades
        echo 'APT::Periodic::Update-Package-Lists "1";' > /etc/apt/apt.conf.d/20auto-upgrades
        echo 'APT::Periodic::Unattended-Upgrade "1";' >> /etc/apt/apt.conf.d/20auto-upgrades
    fi

    # 2. Swap & BBR
    if [ ! -f /swapfile ]; then
        msg "Creating Swap File (Smart Size)..."

        SWAP_MB=$(calculate_swap_size)
        msg "Allocating Swap: ${SWAP_MB}MB"

        if ! fallocate -l "${SWAP_MB}M" /swapfile 2>/dev/null; then
             dd if=/dev/zero of=/swapfile bs=1M count="$SWAP_MB" status=progress
        fi

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
    setup_watchdog

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
