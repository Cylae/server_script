#!/bin/bash
set -euo pipefail

manage_gitea() {
    local name="gitea"
    local sub="git.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing Gitea..."

        # Interactive Credentials
        ask "Enter database password for Gitea (leave empty to generate):" input_pass
        if [ -n "$input_pass" ]; then
             if ! validate_password "$input_pass"; then
                 fatal "Password validation failed."
             fi
             pass="$input_pass"
             save_credential "gitea_db_pass" "$pass"
        else
             pass=$(get_or_create_password "gitea_db_pass")
        fi

        ensure_db "$name" "$name" "$pass"
        local host_ip=$(hostname -I | awk '{print $1}')

        read -r -d '' CONTENT <<EOF
version: "3"
services:
  server:
    image: gitea/gitea:latest
    container_name: gitea
    networks:
      - $DOCKER_NET
    environment:
      - USER_UID=1000
      - USER_GID=1000
      - GITEA__database__DB_TYPE=mysql
      - GITEA__database__HOST=$host_ip:3306
      - GITEA__database__NAME=$name
      - GITEA__database__USER=$name
      - GITEA__database__PASSWD=$pass
      - GITEA__server__SSH_DOMAIN=$sub
      - GITEA__server__SSH_PORT=2222
      - GITEA__server__ROOT_URL=https://$sub/
    restart: always
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:3000:3000"
      - "2222:22"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Gitea" "$sub" "3000" "$CONTENT"

        msg "Gitea Database Credentials:"
        echo -e "   User: ${CYAN}$name${NC}" >&3
        echo -e "   Pass: ${CYAN}$pass${NC}" >&3
    else
        remove_docker_service "$name" "Gitea" "$sub"
    fi
}
