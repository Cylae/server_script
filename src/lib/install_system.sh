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
    # Output format of df -BG: Filesystem 1G-blocks Used Available Use% Mounted on
    # awk 'NR==2 {print $4}' gets the Available column of the second line (e.g., "10G")
    local FREE_DISK=$(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//')

    if [ -z "$FREE_DISK" ]; then
        warn "Could not detect disk space. Skipping check."
    elif [ "$FREE_DISK" -lt 5 ]; then
        fatal "Insufficient Disk Space. Free: ${FREE_DISK}GB. Required: 5GB+"
    fi
}

init_system() {
    check_resources
    msg "Initializing System Infrastructure..."

    export DEBIAN_FRONTEND=noninteractive

    # 1. Basics
    if ! command -v jq &> /dev/null; then
        msg "Installing Basic Dependencies..."
        apt-get update -q && apt-get install -y curl wget git unzip gnupg2 apt-transport-https ca-certificates lsb-release ufw sudo htop apache2-utils fail2ban jq bc iproute2 ncurses-bin pigz unattended-upgrades >/dev/null

        # Configure Unattended Upgrades
        echo 'APT::Periodic::Update-Package-Lists "1";' > /etc/apt/apt.conf.d/20auto-upgrades
        echo 'APT::Periodic::Unattended-Upgrade "1";' >> /etc/apt/apt.conf.d/20auto-upgrades
    fi

    # 2. Swap & BBR
    if [ ! -f /swapfile ]; then
        msg "Creating Swap File (Smart Size)..."

        calculate_swap_size() {
            local RAM_MB=$(free -m | awk '/Mem:/ {print $2}')
            if [ "$RAM_MB" -lt 2048 ]; then
                # < 2GB RAM: 2x RAM
                echo $(( RAM_MB * 2 ))M
            elif [ "$RAM_MB" -le 8192 ]; then
                # 2GB - 8GB RAM: 1x RAM
                echo "${RAM_MB}M"
            else
                # > 8GB RAM: Cap at 4GB
                echo "4096M"
            fi
        }

        SWAP_SIZE=$(calculate_swap_size)
        msg "Allocating Swap: $SWAP_SIZE"

        fallocate -l $SWAP_SIZE /swapfile || dd if=/dev/zero of=/swapfile bs=1M count=${SWAP_SIZE%M}
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
