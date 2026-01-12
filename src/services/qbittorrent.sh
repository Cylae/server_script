manage_qbittorrent() {
    local action=$1
    local name="qbittorrent"
    local pretty_name="qBittorrent"
    local subdomain="qbittorrent"
    local port="8080"

    # qBittorrent has two ports: WebUI (8080) and Torrenting (6881).
    # Nginx proxies 8080.

    if [ "$action" == "install" ]; then
        setup_media_directories
        # Using unified /data mapping for Trash Guides compatibility
        local compose_content=$(cat <<EOF
services:
  qbittorrent:
    image: lscr.io/linuxserver/qbittorrent:latest
    container_name: qbittorrent
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
      - WEBUI_PORT=8080
    volumes:
      - /opt/qbittorrent/config:/config
      - /opt/media:/data
    ports:
      - "8080:8080"
      - "6881:6881"
      - "6881:6881/udp"
    networks:
      - server-net
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 1024M

networks:
  server-net:
    external: true
EOF
)
        deploy_docker_service "$name" "$pretty_name" "$subdomain" "$port" "$compose_content"

        # Open firewall for torrenting port
        ufw allow 6881 >/dev/null 2>&1 || true

    elif [ "$action" == "remove" ]; then
        remove_docker_service "$name" "$pretty_name" "$subdomain" "$port"
        ufw delete allow 6881 >/dev/null 2>&1 || true
    fi
}
