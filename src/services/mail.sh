#!/bin/bash
set -euo pipefail

manage_mail() {
    local name="mail"
    local sub="mail.$DOMAIN"
    if [ "$1" == "install" ]; then
        msg "Installing Mail Server..."
        mkdir -p /opt/mail

        # Resource Check
        local profile=$(cat /etc/cyl_profile)
        local clamav_state=1
        if [ "$profile" == "LOW" ]; then
            clamav_state=0
            msg "Low RAM detected: Disabling ClamAV."
        fi

        CONTENT=$(cat <<EOF
version: '3'
services:
  mailserver:
    image: mailserver/docker-mailserver:latest
    hostname: mail
    domainname: $DOMAIN
    container_name: mailserver
    networks:
      - $DOCKER_NET
    ports:
      - "25:25"
      - "143:143"
      - "587:587"
      - "993:993"
    volumes:
      - ./maildata:/var/mail
      - ./mailstate:/var/mail-state
      - ./maillogs:/var/log/mail
      - ./config:/tmp/docker-mailserver
    environment:
      - ENABLE_SPAMASSASSIN=1
      - ENABLE_CLAMAV=$clamav_state
      - ENABLE_FAIL2BAN=1
      - SSL_TYPE=letsencrypt
    cap_add:
      - NET_ADMIN
    restart: always
  roundcube:
    image: roundcube/roundcubemail:latest
    container_name: roundcube
    networks:
      - $DOCKER_NET
    restart: always
    volumes:
      - ./roundcube/db:/var/www/db
    ports:
      - "127.0.0.1:8081:80"
    environment:
      - ROUNDCUBEMAIL_DEFAULT_HOST=mailserver
      - ROUNDCUBEMAIL_SMTP_SERVER=mailserver
networks:
  $DOCKER_NET:
    external: true
EOF
)
        deploy_docker_service "$name" "Mail Server" "$sub" "8081" "$CONTENT"

        ufw allow 25,587,465,143,993/tcp >/dev/null

        msg "Initializing Mail User..."
        local timeout=60
        local count=0
        until docker exec mailserver setup email list >/dev/null 2>&1; do
            sleep 2
            count=$((count+2))
            if [ $count -ge $timeout ]; then
                fatal "Mail server failed to initialize within $timeout seconds."
            fi
        done

        # Interactive Credentials
        ask "Enter email user (default: postmaster):" input_user
        local mail_user="${input_user:-postmaster}"
        local full_email="$mail_user@$DOMAIN"

        ask "Enter password for $full_email (leave empty to generate):" input_pass
        if [ -n "$input_pass" ]; then
             if ! validate_password "$input_pass"; then
                 fatal "Password validation failed."
             fi
             pass="$input_pass"
             save_credential "postmaster_pass" "$pass"
        else
             pass=$(get_or_create_password "postmaster_pass")
        fi

        # Ensure user exists/updated
        if docker exec mailserver setup email list | grep -q "$mail_user@"; then
             docker exec mailserver setup email update "$full_email" "$pass" >/dev/null 2>&1
        else
             docker exec mailserver setup email add "$full_email" "$pass" >/dev/null 2>&1
        fi
        docker exec mailserver setup config dkim >/dev/null 2>&1

        msg "Mail Account Credentials:"
        echo -e "   User: ${CYAN}$full_email${NC}" >&3
        echo -e "   Pass: ${CYAN}$pass${NC}" >&3
    else
        remove_docker_service "$name" "Mail Server" "$sub"
        ufw delete allow 25,587,465,143,993/tcp >/dev/null 2>&1 || true
    fi
}
