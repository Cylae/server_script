#!/bin/bash
set -euo pipefail

manage_sonarr() {
    local name="sonarr"
    local sub="sonarr.$DOMAIN"
    local port="8989"

    if [ "$1" == "install" ]; then
        CONTENT=$(cat <<EOF
services:
  sonarr:
    image: lscr.io/linuxserver/sonarr:latest
    container_name: sonarr
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - ./config:/config
      - /opt/media:/media # Mount the whole media tree
    ports:
      - "8989:8989"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Sonarr" "$sub" "$port" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Sonarr" "$sub" "$port"
    fi
}
