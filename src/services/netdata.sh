#!/bin/bash
set -euo pipefail

manage_netdata() {
    local name="netdata"
    local sub="netdata.$DOMAIN"

    if [ "$1" == "install" ]; then
        # Check port conflict first
        check_port_conflict "19999" "Netdata"

        CONTENT=$(cat <<EOF
services:
  netdata:
    image: netdata/netdata
    container_name: netdata
    pid: host
    network_mode: host
    restart: unless-stopped
    cap_add:
      - SYS_PTRACE
      - SYS_ADMIN
    security_opt:
      - apparmor:unconfined
    volumes:
      - ./config:/etc/netdata
      - ./lib:/var/lib/netdata
      - ./cache:/var/cache/netdata
      - /etc/passwd:/host/etc/passwd:ro
      - /etc/group:/host/etc/group:ro
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /etc/os-release:/host/etc/os-release:ro
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - NETDATA_CLAIM_TOKEN=
      - NETDATA_CLAIM_URL=
      - NETDATA_CLAIM_ROOMS=
EOF
)
        # Note: Netdata runs on host network, so port "19999" is on host.
        # deploy_docker_service handles nginx config.
        # But `deploy_docker_service` uses `docker compose up`.
        # Netdata running in `network_mode: host` doesn't use the 'server-net' network.
        # This is fine.

        # However, `deploy_docker_service` calls `update_nginx`.
        # `update_nginx` needs a port.
        deploy_docker_service "$name" "Netdata" "$sub" "19999" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Netdata" "$sub"
    fi
}
