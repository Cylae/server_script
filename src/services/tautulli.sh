#!/bin/bash
set -euo pipefail

manage_tautulli() {
    local name="tautulli"
    local sub="tautulli.$DOMAIN"
    local port="8181"

    if [ "$1" == "install" ]; then
        CONTENT=$(cat <<EOF
services:
  tautulli:
    image: lscr.io/linuxserver/tautulli:latest
    container_name: tautulli
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - ./config:/config
    ports:
      - "8181:8181"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Tautulli" "$sub" "$port" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Tautulli" "$sub" "$port"
    fi
}
