manage_overseerr() {
    local action=$1
    local name="overseerr"
    local pretty_name="Overseerr"
    local subdomain="request"
    local port="5055"

    if [ "$action" == "install" ]; then
        setup_media_directories
        # Overseerr uses a different image base (sct/overseerr vs linuxserver), checking PUID env support.
        # sct/overseerr usually runs as root or non-root depending on implementation.
        # LinuxServer image `lscr.io/linuxserver/overseerr` is preferred for consistency with PUID/PGID.
        # Replacing `sct/overseerr` with `lscr.io/linuxserver/overseerr`.

        local compose_content=$(cat <<EOF
services:
  overseerr:
    image: lscr.io/linuxserver/overseerr:latest
    container_name: overseerr
    environment:
      - PUID=\${SUDO_UID:-$(id -u)}
      - PGID=\${SUDO_GID:-$(getent group docker | cut -d: -f3)}
      - TZ=$(cat /etc/timezone)
    volumes:
      - /opt/overseerr/config:/config
    ports:
      - "5055:5055"
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
