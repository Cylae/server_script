#!/bin/bash
set -euo pipefail

manage_glpi() {
    local name="glpi"
    local sub="support.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing GLPI (IT Ticketing)..."

        # Interactive Credentials
        ask "Enter database password for GLPI (leave empty to generate):" input_pass
        if [ -n "$input_pass" ]; then
             if ! validate_password "$input_pass"; then
                 fatal "Password validation failed."
             fi
             pass="$input_pass"
             save_credential "glpi_db_pass" "$pass"
        else
             pass=$(get_or_create_password "glpi_db_pass")
        fi

        ensure_db "$name" "$name" "$pass"
        local host_ip=$(docker network inspect $DOCKER_NET | jq -r '.[0].IPAM.Config[0].Gateway')

        # We use diouxx/glpi for simplicity
        CONTENT=$(cat <<EOF
services:
  glpi:
    image: diouxx/glpi:latest
    container_name: glpi
    restart: always
    networks:
      - $DOCKER_NET
    ports:
      - 127.0.0.1:8082:80
    volumes:
      - ./data:/var/www/html/glpi
    environment:
      - TIMEZONE=Europe/Paris
      - MARIADB_HOST=$host_ip
      - MARIADB_PORT=3306
      - MARIADB_DATABASE=$name
      - MARIADB_USER=$name
      - MARIADB_PASSWORD=$pass
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "GLPI" "$sub" "8082" "$CONTENT"

        msg "GLPI Default Credentials:"
        echo -e "   User: ${CYAN}glpi${NC}" >&3
        echo -e "   Pass: ${CYAN}glpi${NC}" >&3
        warn "Please change the default password immediately!"
    else
        remove_docker_service "$name" "GLPI" "$sub"
    fi
}
