#!/bin/bash
# lib/services.sh - Service definitions

# Depends on lib/utils.sh and lib/core.sh

manage_gitea() {
    local name="gitea"
    local sub="git.$DOMAIN"
    if [ "$1" == "install" ]; then
        # Interactive Credentials
        ask_credential "Gitea (Database)" "gitea" "gitea_db"
        local db_user="$SET_USER"
        local db_pass="$SET_PASS"

        # We don't ask for admin user here because Gitea has a web setup wizard.
        # But we ensure DB is ready.
        ensure_db "$name" "$db_user" "$db_pass"
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
      - GITEA__database__USER=$db_user
      - GITEA__database__PASSWD=$db_pass
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

        show_service_report "Gitea" "https://$sub" "(Web Setup)" "(Web Setup)"
    else
        remove_docker_service "$name" "Gitea" "$sub"
    fi
}

manage_nextcloud() {
    local name="nextcloud"
    local sub="cloud.$DOMAIN"
    if [ "$1" == "install" ]; then
        ask_credential "Nextcloud (Database)" "nextcloud" "nextcloud_db"
        local db_user="$SET_USER"
        local db_pass="$SET_PASS"

        ensure_db "$name" "$db_user" "$db_pass"
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
      - MYSQL_PASSWORD=$db_pass
      - MYSQL_DATABASE=$name
      - MYSQL_USER=$db_user
      - MYSQL_HOST=$host_ip
      - NEXTCLOUD_TRUSTED_DOMAINS=$sub
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Nextcloud" "$sub" "8080" "$CONTENT"

        msg "Waiting for Nextcloud to initialize..."
        until docker exec -u www-data nextcloud_app php occ status >/dev/null 2>&1; do sleep 2; done
        docker exec -u www-data nextcloud_app php occ config:system:set trusted_proxies 0 --value="127.0.0.1" >/dev/null 2>&1
        docker exec -u www-data nextcloud_app php occ config:system:set overwriteprotocol --value="https" >/dev/null 2>&1

        show_service_report "Nextcloud" "https://$sub" "(Web Setup)" "(Web Setup)"
    else
        remove_docker_service "$name" "Nextcloud" "$sub"
    fi
}

manage_vaultwarden() {
    local name="vaultwarden"
    local sub="pass.$DOMAIN"
    if [ "$1" == "install" ]; then
        read -r -d '' CONTENT <<EOF
version: '3'
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: always
    environment:
      - SIGNUPS_ALLOWED=true
    volumes:
      - ./data:/data
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8082:80"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Vaultwarden" "$sub" "8082" "$CONTENT"

        show_service_report "Vaultwarden" "https://$sub" "(Web Setup)" "(Web Setup)"
    else
        remove_docker_service "$name" "Vaultwarden" "$sub"
    fi
}

manage_uptimekuma() {
    local name="uptimekuma"
    local sub="status.$DOMAIN"
    if [ "$1" == "install" ]; then
        read -r -d '' CONTENT <<EOF
version: '3'
services:
  uptime-kuma:
    image: louislam/uptime-kuma:1
    container_name: uptime-kuma
    restart: always
    volumes:
      - ./data:/app/data
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:3001:3001"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "Uptime Kuma" "$sub" "3001" "$CONTENT"

        show_service_report "Uptime Kuma" "https://$sub" "(Web Setup)" "(Web Setup)"
    else
        remove_docker_service "$name" "Uptime Kuma" "$sub"
    fi
}

manage_wireguard() {
    local name="wireguard"
    local sub="vpn.$DOMAIN"
    if [ "$1" == "install" ]; then
        # Explicitly ask for WG Password which is used in ENV
        ask_credential "WireGuard" "admin" "wg"
        local wg_pass="$SET_PASS"

        local host_ip=$(curl -s https://api.ipify.org)

        read -r -d '' CONTENT <<EOF
version: "3.8"
services:
  wg-easy:
    environment:
      - WG_HOST=$host_ip
      - PASSWORD=$wg_pass
      - WG_PORT=51820
      - WG_DEFAULT_ADDRESS=10.8.0.x
      - WG_DEFAULT_DNS=1.1.1.1
      - WG_ALLOWED_IPS=0.0.0.0/0
      - WG_PERSISTENT_KEEPALIVE=25
    image: ghcr.io/wg-easy/wg-easy
    container_name: wg-easy
    volumes:
      - ./data:/etc/wireguard
    ports:
      - "51820:51820/udp"
      - "127.0.0.1:51821:51821/tcp"
    restart: unless-stopped
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    sysctls:
      - net.ipv4.ip_forward=1
      - net.ipv4.conf.all.src_valid_mark=1
    networks:
      - $DOCKER_NET
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "WireGuard" "$sub" "51821" "$CONTENT"
        ufw allow 51820/udp >/dev/null

        show_service_report "WireGuard" "https://$sub" "(None)" "$wg_pass"
    else
        remove_docker_service "$name" "WireGuard" "$sub"
        ufw delete allow 51820/udp >/dev/null 2>&1 || true
    fi
}

manage_filebrowser() {
    local name="filebrowser"
    local sub="files.$DOMAIN"
    if [ "$1" == "install" ]; then
        mkdir -p /opt/$name
        touch /opt/$name/filebrowser.db
        echo '{"port": 80, "baseURL": "", "address": "", "log": "stdout", "database": "/database.db", "root": "/srv"}' > /opt/$name/settings.json

        read -r -d '' CONTENT <<EOF
version: '3'
services:
  filebrowser:
    image: filebrowser/filebrowser
    container_name: filebrowser
    restart: always
    volumes:
      - /var/www:/srv
      - ./filebrowser.db:/database.db
      - ./settings.json:/.filebrowser.json
    networks:
      - $DOCKER_NET
    ports:
      - "127.0.0.1:8083:80"
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "FileBrowser" "$sub" "8083" "$CONTENT"

        show_service_report "FileBrowser" "https://$sub" "admin" "admin"
    else
        remove_docker_service "$name" "FileBrowser" "$sub"
    fi
}

manage_yourls() {
    local name="yourls"
    local sub="x.$DOMAIN"
    if [ "$1" == "install" ]; then
        ask_credential "YOURLS (Database & Admin)" "yourls" "yourls_db"
        local db_user="$SET_USER"
        local db_pass="$SET_PASS"

        # We reuse the same password for admin login in this implementation
        local admin_user="admin"
        local admin_pass="$db_pass"

        ensure_db "$name" "$db_user" "$db_pass"
        local host_ip=$(hostname -I | awk '{print $1}')
        local cookie=$(openssl rand -hex 16)

        read -r -d '' CONTENT <<EOF
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
      - YOURLS_DB_USER=$db_user
      - YOURLS_DB_PASS=$db_pass
      - YOURLS_DB_NAME=$name
      - YOURLS_SITE=https://$sub
      - YOURLS_USER=$admin_user
      - YOURLS_PASS=$admin_pass
      - YOURLS_COOKIEKEY=$cookie
networks:
  $DOCKER_NET:
    external: true
EOF
        deploy_docker_service "$name" "YOURLS" "$sub" "8084" "$CONTENT"

        show_service_report "YOURLS" "https://$sub/admin" "$admin_user" "$admin_pass"
    else
        remove_docker_service "$name" "YOURLS" "$sub"
    fi
}

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

        read -r -d '' CONTENT <<EOF
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
        deploy_docker_service "$name" "Mail Server" "$sub" "8081" "$CONTENT"

        ufw allow 25,587,465,143,993/tcp >/dev/null

        msg "Initializing Mail User..."
        until docker exec mailserver setup email list >/dev/null 2>&1; do sleep 2; done

        ask_credential "Mail Server" "postmaster@$DOMAIN" "postmaster"
        local mail_user="$SET_USER"
        local mail_pass="$SET_PASS"

        # Ensure user exists/updated
        if docker exec mailserver setup email list | grep -q "$mail_user"; then
             docker exec mailserver setup email update "$mail_user" "$mail_pass" >/dev/null 2>&1
        else
             docker exec mailserver setup email add "$mail_user" "$mail_pass" >/dev/null 2>&1
        fi
        docker exec mailserver setup config dkim >/dev/null 2>&1

        show_service_report "Mail Server" "https://$sub" "$mail_user" "$mail_pass"
    else
        remove_docker_service "$name" "Mail Server" "$sub"
        ufw delete allow 25,587,465,143,993/tcp >/dev/null 2>&1 || true
    fi
}

manage_netdata() {
    local name="netdata"
    local sub="netdata.$DOMAIN"

    if [ "$1" == "install" ]; then
        msg "Installing Netdata..."
        mkdir -p "/opt/$name"
        # Using bind mounts to ensure we can back them up
        mkdir -p "/opt/$name/config" "/opt/$name/lib" "/opt/$name/cache"

        # We run this one manually as it requires many host mounts
        # Port check before running
        safety_check "19999"

        docker run -d --name=netdata --pid=host --network=$DOCKER_NET -p 127.0.0.1:19999:19999 \
          -v "/opt/$name/config:/etc/netdata" \
          -v "/opt/$name/lib:/var/lib/netdata" \
          -v "/opt/$name/cache:/var/cache/netdata" \
          -v /etc/passwd:/host/etc/passwd:ro -v /etc/group:/host/etc/group:ro -v /proc:/host/proc:ro \
          -v /sys:/host/sys:ro -v /etc/os-release:/host/etc/os-release:ro -v /var/run/docker.sock:/var/run/docker.sock \
          --restart always --cap-add SYS_PTRACE --security-opt apparmor=unconfined netdata/netdata

        update_nginx "$sub" "19999" "proxy"
        enable_ssl "$sub"

        show_service_report "Netdata" "https://$sub" "(None)" "(None)"
    elif [ "$1" == "remove" ]; then
        docker stop netdata && docker rm netdata

        ask "Do you want to PERMANENTLY DELETE data for Netdata? (y/n):" confirm_delete
        if [[ "$confirm_delete" == "y" ]]; then
            rm -rf "/opt/$name"
        fi

        rm -f "/etc/nginx/sites-enabled/$sub"
        success "Netdata Removed"
    fi
}

manage_portainer() {
    local name="portainer"
    local sub="portainer.$DOMAIN"

    if [ "$1" == "install" ]; then
        msg "Installing Portainer..."
        mkdir -p "/opt/$name/data"

        safety_check "9000"

        docker run -d -p 127.0.0.1:9000:9000 --name=portainer --network $DOCKER_NET --restart=always \
        -v /var/run/docker.sock:/var/run/docker.sock -v "/opt/$name/data:/data" portainer/portainer-ce

        update_nginx "$sub" "9000" "proxy"
        enable_ssl "$sub"

        show_service_report "Portainer" "https://$sub" "(Web Setup)" "(Web Setup)"
    elif [ "$1" == "remove" ]; then
        docker stop portainer && docker rm portainer

        ask "Do you want to PERMANENTLY DELETE data for Portainer? (y/n):" confirm_delete
        if [[ "$confirm_delete" == "y" ]]; then
            rm -rf "/opt/$name"
        fi

        rm -f "/etc/nginx/sites-enabled/$sub"
        success "Portainer Removed"
    fi
}

manage_ftp() {
    if [ "$1" == "install" ]; then
        msg "Installing FTP (vsftpd)..."
        apt-get install -y vsftpd >/dev/null
        cp /etc/vsftpd.conf /etc/vsftpd.conf.bak 2>/dev/null

        local pasv_addr=$(curl -s https://api.ipify.org)
        cat <<EOF > /etc/vsftpd.conf
listen=NO
listen_ipv6=YES
anonymous_enable=NO
local_enable=YES
write_enable=YES
dirmessage_enable=YES
use_localtime=YES
xferlog_enable=YES
connect_from_port_20=YES
chroot_local_user=YES
secure_chroot_dir=/var/run/vsftpd/empty
pam_service_name=vsftpd
rsa_cert_file=/etc/ssl/certs/ssl-cert-snakeoil.pem
rsa_private_key=/etc/ssl/private/ssl-cert-snakeoil.key
ssl_enable=YES
allow_anon_ssl=NO
ssl_tlsv1=YES
pasv_min_port=40000
pasv_max_port=50000
allow_writeable_chroot=YES
pasv_address=$pasv_addr
EOF
        ufw allow 20,21,990/tcp >/dev/null
        ufw allow 40000:50000/tcp >/dev/null
        systemctl restart vsftpd

        # Ensure user
        ask_credential "FTP User" "cyluser" "ftp"
        local ftp_user="$SET_USER"
        local ftp_pass="$SET_PASS"

        if ! id "$ftp_user" &>/dev/null; then
            useradd -m -d /var/www -s /bin/bash "$ftp_user"
            usermod -a -G www-data "$ftp_user"
            chown -R "$ftp_user":www-data /var/www
        fi

        # Always set password to ensure consistency
        echo "$ftp_user:$ftp_pass" | chpasswd

        show_service_report "FTP" "ftp://$pasv_addr" "$ftp_user" "$ftp_pass"
    elif [ "$1" == "remove" ]; then
        apt-get remove -y vsftpd >/dev/null
        ufw delete allow 20/tcp >/dev/null 2>&1 || true
        success "FTP Removed"
    fi
}
