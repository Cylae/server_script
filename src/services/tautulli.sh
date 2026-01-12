manage_tautulli() {
    local action=$1
    local name="tautulli"
    local pretty_name="Tautulli"
    local subdomain="tautulli"
    local port="8181"

    if [ "$action" == "install" ]; then
        local compose_content=$(cat <<EOF
services:
  tautulli:
    image: lscr.io/linuxserver/tautulli:latest
    container_name: tautulli
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
    volumes:
      - /opt/tautulli/config:/config
    ports:
      - "8181:8181"
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
