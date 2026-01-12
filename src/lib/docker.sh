#!/bin/bash
set -euo pipefail

# ==============================================================================
# DOCKER MODULE
# Docker helpers
# ==============================================================================

DOCKER_NET="server-net"

init_docker() {
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

    # Configure Log Rotation
    if [ ! -f /etc/docker/daemon.json ]; then
        msg "Configuring Docker Log Rotation..."
        mkdir -p /etc/docker
        cat <<EOF > /etc/docker/daemon.json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
EOF
        systemctl restart docker || warn "Failed to restart Docker after config."
    fi
}

check_port_conflict() {
    local port=$1
    local name=$2

    # Check if port is in use using ss
    # -t: tcp, -u: udp, -l: listening, -n: numeric
    # We look for specific port binding.
    # Pattern: ":PORT " or ":PORT$"
    if ss -tuln | awk '{print $5}' | grep -E ":$port$" >/dev/null 2>&1; then
        warn "Port $port is currently in use."
        ask "Is this expected (e.g., updating existing service)? (y/n):" confirm
        if [[ "$confirm" != "y" ]]; then
            fatal "Aborting installation of $name due to port conflict on $port."
        fi
    fi
}

deploy_docker_service() {
    local name=$1
    local pretty_name=$2
    local subdomain=$3
    local port=$4
    local docker_compose_content=$5

    msg "Installing $pretty_name..."

    # Check for port conflicts if a port is exposed
    if [ -n "$port" ]; then
        check_port_conflict "$port" "$pretty_name"
    fi

    mkdir -p "/opt/$name"
    echo "$docker_compose_content" > "/opt/$name/docker-compose.yml"

    # Robust deployment: pull first to minimize downtime, then up -d
    # 'up -d' handles recreation if config changed
    cd "/opt/$name"
    docker compose pull --quiet || warn "Failed to pull images for $name, trying to build/run anyway..."
    docker compose up -d

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
        if [ -f "/opt/$name/docker-compose.yml" ]; then
            # We use || true to ensure script doesn't fail if containers are already gone/borked
            cd "/opt/$name" && docker compose down || true
        fi
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
