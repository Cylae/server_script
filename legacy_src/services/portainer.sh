#!/bin/bash
set -euo pipefail

manage_portainer() {
    local name="portainer"
    local sub="portainer.$DOMAIN"

    if [ "$1" == "install" ]; then
        # Migrate legacy container if it exists and isn't managed by compose
        if docker ps -a --format '{{.Names}}' | grep -q "^portainer$"; then
             if ! docker inspect portainer | grep -q "com.docker.compose.project"; then
                 msg "Migrating legacy Portainer container to Docker Compose..."
                 docker stop portainer >/dev/null 2>&1 || true
                 docker rm portainer >/dev/null 2>&1 || true
             fi
        fi

        CONTENT=$(cat <<EOF
services:
  portainer:
    image: portainer/portainer-ce
    container_name: portainer
    restart: always
    ports:
      - "127.0.0.1:9000:9000"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./data:/data
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Portainer" "$sub" "9000" "$CONTENT"

    elif [ "$1" == "remove" ]; then
        # Handle legacy container removal
        if docker ps -a --format '{{.Names}}' | grep -q "^portainer$"; then
             if ! docker inspect portainer | grep -q "com.docker.compose.project"; then
                  docker stop portainer >/dev/null 2>&1 || true
                  docker rm portainer >/dev/null 2>&1 || true
             fi
        fi

        remove_docker_service "$name" "Portainer" "$sub" "9000"
    fi
}
