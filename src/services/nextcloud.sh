#!/bin/bash

manage_nextcloud() {
    local name="nextcloud"
    local sub="cloud.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing Nextcloud..."

        # Interactive Credentials
        ask "Enter database password for Nextcloud (leave empty to generate):" input_pass
        if [ -n "$input_pass" ]; then
             if ! validate_password "$input_pass"; then
                 fatal "Password validation failed."
             fi
             pass="$input_pass"
             save_credential "nextcloud_db_pass" "$pass"
        else
             pass=$(get_or_create_password "nextcloud_db_pass")
        fi

        ensure_db "$name" "$name" "$pass"
        local host_ip=$(hostname -I | awk '{print $1}')

        read -r -d '' CONTENT <<EOF
version: '2'
services:
  app:
    image: nextcloud
    container_name: nextcloud_app
    restart: always
    networks:
      - $DOCKER_NET
    ports:
      - 127.0.0.1:8080:80
    volumes:
      - ./nextcloud:/var/www/html
    environment:
      - MYSQL_PASSWORD=$pass
      - MYSQL_DATABASE=$name
      - MYSQL_USER=$name
      - MYSQL_HOST=$host_ip
      - NEXTCLOUD_TRUSTED_DOMAINS=$sub
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Nextcloud" "$sub" "8080" "$CONTENT"

        msg "Waiting for Nextcloud to initialize..."
        local timeout=120
        local count=0
        until docker exec -u www-data nextcloud_app php occ status >/dev/null 2>&1; do
             sleep 2
             count=$((count+2))
             if [ $count -ge $timeout ]; then
                 fatal "Nextcloud failed to initialize within $timeout seconds."
             fi
        done
        docker exec -u www-data nextcloud_app php occ config:system:set trusted_proxies 0 --value="127.0.0.1" >/dev/null 2>&1
        docker exec -u www-data nextcloud_app php occ config:system:set overwriteprotocol --value="https" >/dev/null 2>&1
    else
        remove_docker_service "$name" "Nextcloud" "$sub"
    fi
}
