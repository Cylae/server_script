#!/bin/bash
set -euo pipefail

manage_plex() {
    local name="plex"
    local sub="plex.$DOMAIN"
    local port="32400"

    if [ "$1" == "install" ]; then
        # Plex Claim Token is optional but recommended.
        # We can ask for it or let user handle it.
        # For seamless integration, we'll just install.

        # Note: Plex requires host networking for best results (DLNA, discovery),
        # but the prompt asks to integrate into the ecosystem (Nginx).
        # We can expose 32400 and proxy it.
        # Using host networking complicates the proxy setup via docker-compose usually,
        # but 'network_mode: host' allows it.
        # However, our 'deploy_docker_service' uses a custom network.
        # Plex usually works fine bridged if you specify ADVERTISE_IP.

        CONTENT=$(cat <<EOF
services:
  plex:
    image: lscr.io/linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID=1000
      - PGID=1000
      - VERSION=docker
    volumes:
      - ./config:/config
      - /opt/media:/media
    ports:
      - "32400:32400"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Plex Media Server" "$sub" "$port" "$CONTENT"

        # Plex needs some special Nginx handling sometimes?
        # Standard proxy usually works for web access.

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Plex Media Server" "$sub" "$port"
    fi
}
