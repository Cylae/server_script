#!/bin/bash
set -euo pipefail

manage_radarr() {
    local name="radarr"
    local sub="radarr.$DOMAIN"
    local port="7878"

    if [ "$1" == "install" ]; then
        CONTENT=$(cat <<EOF
services:
  radarr:
    image: lscr.io/linuxserver/radarr:latest
    container_name: radarr
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - ./config:/config
      - /opt/media:/media
    ports:
      - "7878:7878"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Radarr" "$sub" "$port" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Radarr" "$sub" "$port"
    fi
}
