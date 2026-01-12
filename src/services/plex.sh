deploy_plex() {
    local name="plex"
    local pretty_name="Plex Media Server"
    local subdomain="plex"
    local port="32400"

    setup_media_directories

    # Using standard bridge mode for Plex to keep it simple and compatible with our architecture.
    # Map /opt/media:/data for unified access.

    local compose_content_bridge=$(cat <<EOF
services:
  plex:
    image: linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
      - VERSION=docker
    volumes:
      - /opt/plex/config:/config
      - /opt/media:/data
    ports:
      - "32400:32400"
    networks:
      - server-net
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 2048M

networks:
  server-net:
    external: true
EOF
)
    deploy_docker_service "$name" "$pretty_name" "$subdomain" "$port" "$compose_content_bridge"
}

manage_plex() {
    local action=$1
    if [ "$action" == "install" ]; then
        deploy_plex
    elif [ "$action" == "remove" ]; then
        remove_docker_service "plex" "Plex Media Server" "plex" "32400"
    fi
}
