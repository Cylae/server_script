#!/bin/bash
set -euo pipefail

manage_yourls() {
    local name="yourls"
    local sub="x.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing YOURLS..."

        # Interactive Credentials
        ask "Enter password for YOURLS admin (leave empty to generate):" input_pass
        if [ -n "$input_pass" ]; then
             if ! validate_password "$input_pass"; then
                 fatal "Password validation failed."
             fi
             pass="$input_pass"
             save_credential "yourls_pass" "$pass"
        else
             pass=$(get_or_create_password "yourls_pass")
        fi

        ensure_db "$name" "$name" "$pass"
        local host_ip=$(docker network inspect $DOCKER_NET | jq -r '.[0].IPAM.Config[0].Gateway')
        local cookie=$(openssl rand -hex 16)

        # NOTE: Mapping /var/www/html to local 'data' dir to persist plugins/config
        CONTENT=$(cat <<EOF
version: '3'
services:
  yourls:
    image: yourls:latest
    container_name: yourls
    restart: always
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8084:80"
    volumes:
      - ./data:/var/www/html
    environment:
      - YOURLS_DB_HOST=$host_ip
      - YOURLS_DB_USER=$name
      - YOURLS_DB_PASS=$pass
      - YOURLS_DB_NAME=$name
      - YOURLS_SITE=https://$sub
      - YOURLS_USER=admin
      - YOURLS_PASS=$pass
      - YOURLS_COOKIEKEY=$cookie
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "YOURLS" "$sub" "8084" "$CONTENT"
        msg "YOURLS Admin Password: $pass"
    else
        remove_docker_service "$name" "YOURLS" "$sub"
    fi
}
