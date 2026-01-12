manage_prowlarr() {
    local action=$1
    local name="prowlarr"
    local pretty_name="Prowlarr"
    local subdomain="prowlarr"
    local port="9696"

    if [ "$action" == "install" ]; then
        setup_media_directories
        local compose_content=$(cat <<EOF
services:
  prowlarr:
    image: lscr.io/linuxserver/prowlarr:latest
    container_name: prowlarr
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
    volumes:
      - /opt/prowlarr/config:/config
    ports:
      - "9696:9696"
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
