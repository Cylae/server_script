#!/bin/bash
set -euo pipefail

manage_wireguard() {
    local name="wireguard"
    local sub="vpn.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing WireGuard (WG-Easy)..."

        ask "Enter WG Password (leave empty for auto/reuse):" WGPASS
        if [ -n "$WGPASS" ]; then
             if ! validate_password "$WGPASS"; then
                 fatal "Password validation failed."
             fi
             save_credential "wg_pass" "$WGPASS"
        else
             WGPASS=$(get_or_create_password "wg_pass")
        fi

        local host_ip=$(curl -s https://api.ipify.org)

        CONTENT=$(cat <<EOF
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
)
        deploy_docker_service "$name" "WireGuard" "$sub" "51821" "$CONTENT"
        ufw allow 51820/udp >/dev/null
        msg "WireGuard Password: $WGPASS"
    else
        remove_docker_service "$name" "WireGuard" "$sub"
        ufw delete allow 51820/udp >/dev/null 2>&1 || true
    fi
}
