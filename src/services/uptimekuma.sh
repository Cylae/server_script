#!/bin/bash
set -euo pipefail

manage_uptimekuma() {
    local name="uptimekuma"
    local sub="status.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing Uptime Kuma..."

        read -r -d '' CONTENT <<EOF
version: '3'
services:
  uptime-kuma:
    image: louislam/uptime-kuma:1
    container_name: uptime-kuma
    restart: always
    volumes:
      - ./data:/app/data
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:3001:3001"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Uptime Kuma" "$sub" "3001" "$CONTENT"
    else
        remove_docker_service "$name" "Uptime Kuma" "$sub"
    fi
}
