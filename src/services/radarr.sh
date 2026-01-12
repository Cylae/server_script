manage_radarr() {
    local action=$1
    local name="radarr"
    local pretty_name="Radarr"
    local subdomain="radarr"
    local port="7878"

    if [ "$action" == "install" ]; then
        setup_media_directories
        # Using unified /data mapping for Trash Guides compatibility
        local compose_content=$(cat <<EOF
services:
  radarr:
    image: lscr.io/linuxserver/radarr:latest
    container_name: radarr
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
    volumes:
      - /opt/radarr/config:/config
      - /opt/media:/data
    ports:
      - "7878:7878"
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

    elif [ "$action" == "remove" ]; then
        remove_docker_service "$name" "$pretty_name" "$subdomain" "$port"
    fi
}
