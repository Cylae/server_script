#!/bin/bash
set -euo pipefail

manage_qbittorrent() {
    local name="qbittorrent"
    local sub="qbittorrent.$DOMAIN"
    local port="8080" # WebUI port

    if [ "$1" == "install" ]; then
        # Standard qBittorrent WebUI port is 8080
        # Torrenting port is usually 6881 (TCP/UDP)

        # We need to get the password or set a default.
        # qBittorrent >= 4.6.1 generates a random password on first run and prints to stdout.
        # We can set 'WEBUI_PORT=8080' env var.

        # For simplicity in this setup, we rely on the user checking logs or setting it via UI.
        # But we can try to use linuxserver/qbittorrent which has consistent behavior.

        CONTENT=$(cat <<EOF
services:
  qbittorrent:
    image: lscr.io/linuxserver/qbittorrent:latest
    container_name: qbittorrent
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
      - WEBUI_PORT=8080
    volumes:
      - ./config:/config
      - /opt/media/downloads:/downloads
      - /opt/media/watch:/watch
    ports:
      - "8080:8080"
      - "6881:6881"
      - "6881:6881/udp"
    restart: unless-stopped
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "qBittorrent" "$sub" "$port" "$CONTENT"

        msg "Info: qBittorrent default credentials are usually printed in logs."
        msg "Run 'docker logs qbittorrent' to see the temporary password if applicable."

    elif [ "$1" == "remove" ]; then
        remove_docker_service "$name" "qBittorrent" "$sub" "$port"
    fi
}
