#!/bin/bash
set -euo pipefail

manage_jackett() {
    local name="jackett"
    local sub="jackett.$DOMAIN"
    local port="9117"

    if [ "$1" == "install" ]; then
        CONTENT=$(cat <<EOF
services:
  jackett:
    image: lscr.io/linuxserver/jackett:latest
    container_name: jackett
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - ./config:/config
      - /opt/media/downloads:/downloads
    ports:
      - "9117:9117"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Jackett" "$sub" "$port" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Jackett" "$sub" "$port"
    fi
}
