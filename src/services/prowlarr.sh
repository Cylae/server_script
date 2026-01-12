#!/bin/bash
set -euo pipefail

manage_prowlarr() {
    local name="prowlarr"
    local sub="prowlarr.$DOMAIN"
    local port="9696"

    if [ "$1" == "install" ]; then
        CONTENT=$(cat <<EOF
services:
  prowlarr:
    image: lscr.io/linuxserver/prowlarr:latest
    container_name: prowlarr
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
    volumes:
      - ./config:/config
    ports:
      - "9696:9696"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Prowlarr" "$sub" "$port" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "Prowlarr" "$sub" "$port"
    fi
}
