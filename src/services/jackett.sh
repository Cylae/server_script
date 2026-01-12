manage_jackett() {
    local action=$1
    local name="jackett"
    local pretty_name="Jackett"
    local subdomain="jackett"
    local port="9117"

    if [ "$action" == "install" ]; then
        setup_media_directories
        # Using unified /data mapping for Trash Guides compatibility
        local compose_content=$(cat <<EOF
services:
  jackett:
    image: lscr.io/linuxserver/jackett:latest
    container_name: jackett
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
      - AUTO_UPDATE=true
    volumes:
      - /opt/jackett/config:/config
      - /opt/media:/data
    ports:
      - "9117:9117"
    networks:
      - server-net
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 512M

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
