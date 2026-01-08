#!/bin/bash

# ==============================================================================
# PROXY MODULE
# Nginx and Certbot helpers
# ==============================================================================

update_nginx() {
    local sub=$1
    local port=$2
    local type=$3 # proxy or php
    local root=${4:-}

    # Security Headers
    local SEC_HEADERS='
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    '

    rm -f "/etc/nginx/sites-enabled/default"

    cat <<EOF > "/etc/nginx/sites-available/$sub"
server {
    listen 80;
    server_name $sub;
    client_max_body_size 10G;
    $SEC_HEADERS
EOF

    if [ "$type" == "proxy" ]; then
        cat <<EOF >> "/etc/nginx/sites-available/$sub"
    location / {
        proxy_pass http://127.0.0.1:$port;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF
    elif [ "$type" == "php" ] || [ "$type" == "dashboard" ]; then
        PHP_SOCK=$(find /run/php/ -name "php*-fpm.sock" | head -n 1)
        cat <<EOF >> "/etc/nginx/sites-available/$sub"
    root $root;
    index index.php index.html;
EOF
        if [ "$type" == "dashboard" ]; then
            echo '    location / { try_files $uri $uri/ =404; auth_basic "Restricted"; auth_basic_user_file /etc/nginx/.htpasswd; }' >> "/etc/nginx/sites-available/$sub"
        else
             echo '    location / { try_files $uri $uri/ =404; }' >> "/etc/nginx/sites-available/$sub"
        fi

        cat <<EOF >> "/etc/nginx/sites-available/$sub"
    location ~ \.php$ {
        include snippets/fastcgi-php.conf;
        fastcgi_pass unix:$PHP_SOCK;
    }
}
EOF
    fi

    ln -sf "/etc/nginx/sites-available/$sub" "/etc/nginx/sites-enabled/"
}

sync_infrastructure() {
    msg "Syncing Infrastructure (Nginx & SSL)..."

    systemctl reload nginx

    # Get all domains from sites-enabled
    local domains=""

    # Enable nullglob to handle empty directory
    shopt -s nullglob
    for conf in /etc/nginx/sites-enabled/*; do
        # Ensure it is a file
        [ -f "$conf" ] || continue

        local d=$(grep "server_name" "$conf" 2>/dev/null | awk '{print $2}' | tr -d ';' || true)
        if [ -n "$d" ] && [ "$d" != "_" ]; then
            domains="$domains -d $d"
        fi
    done
    shopt -u nullglob

    if [ -n "$domains" ]; then
        msg "Requesting SSL certificates for: $domains"
        certbot --nginx --non-interactive --agree-tos -m "$EMAIL" --expand $domains
    fi

    success "Infrastructure Synced."
}
