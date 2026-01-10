#!/bin/bash
set -euo pipefail

manage_vaultwarden() {
    local name="vaultwarden"
    local sub="pass.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing Vaultwarden..."
        # Vaultwarden doesn't use external DB in this config, so no password prompt needed for DB.

        CONTENT=$(cat <<EOF
version: '3'
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: always
    environment:
      - SIGNUPS_ALLOWED=true
    volumes:
      - ./data:/data
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8082:80"
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Vaultwarden" "$sub" "8082" "$CONTENT"
    else
        remove_docker_service "$name" "Vaultwarden" "$sub"
    fi
}
