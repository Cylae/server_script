manage_sonarr() {
    local action=$1
    local name="sonarr"
    local pretty_name="Sonarr"
    local subdomain="sonarr"
    local port="8989"

    if [ "$action" == "install" ]; then
        setup_media_directories
        # Using unified /data mapping for Trash Guides compatibility
        local compose_content=$(cat <<EOF
services:
  sonarr:
    image: lscr.io/linuxserver/sonarr:latest
    container_name: sonarr
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
    volumes:
      - /opt/sonarr/config:/config
      - /opt/media:/data
    ports:
      - "8989:8989"
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
