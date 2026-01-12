#!/bin/bash
set -euo pipefail

manage_overseerr() {
    local name="overseerr"
    local sub="overseerr.$DOMAIN"
    local port="5055"

    if [ "$1" == "install" ]; then
        CONTENT=$(cat <<EOF
services:
  overseerr:
    image: lscr.io/linuxserver/overseerr:latest
    container_name: overseerr
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - ./config:/config
    ports:
      - "5055:5055"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Overseerr" "$sub" "$port" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Overseerr" "$sub" "$port"
    fi
}
