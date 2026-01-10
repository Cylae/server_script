#!/bin/bash
set -euo pipefail

manage_filebrowser() {
    local name="filebrowser"
    local sub="files.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing FileBrowser..."
        mkdir -p /opt/$name
        touch /opt/$name/filebrowser.db
        echo '{"port": 80, "baseURL": "", "address": "", "log": "stdout", "database": "/database.db", "root": "/srv"}' > /opt/$name/settings.json

        read -r -d '' CONTENT <<EOF
version: '3'
services:
  filebrowser:
    image: filebrowser/filebrowser
    container_name: filebrowser
    restart: always
    volumes:
      - /var/www:/srv
      - ./filebrowser.db:/database.db
      - ./settings.json:/.filebrowser.json
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8083:80"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "FileBrowser" "$sub" "8083" "$CONTENT"
        success "Default login: admin/admin"
    else
        remove_docker_service "$name" "FileBrowser" "$sub"
    fi
}
