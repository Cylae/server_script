#!/bin/bash
set -euo pipefail

# ==============================================================================
# UTILS MODULE
# General utilities (Backup, Restore, System Tuning, etc)
# ==============================================================================

manage_backup() {
    msg "Starting Backup..."
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)

    # DB Dump
    if command -v mysqldump >/dev/null; then
        # Check if DB_ROOT_PASS is set, otherwise try to load it
        if [ -z "${DB_ROOT_PASS:-}" ]; then
             if grep -q "mysql_root_password" "$AUTH_FILE"; then
                 DB_ROOT_PASS=$(get_auth_value "mysql_root_password")
             fi
        fi

        if [ -n "${DB_ROOT_PASS:-}" ]; then
             mysqldump -u root --password="$DB_ROOT_PASS" --all-databases > "$BACKUP_DIR/db_$TIMESTAMP.sql"
        else
             warn "Database root password not found. Skipping DB backup."
        fi
    fi

    # Files
    if command -v pigz >/dev/null; then
        tar --use-compress-program=pigz -cf "$BACKUP_DIR/files_$TIMESTAMP.tar.gz" /opt /var/www /etc/nginx/sites-available /root/.auth_details 2>/dev/null
    else
        tar -czf "$BACKUP_DIR/files_$TIMESTAMP.tar.gz" /opt /var/www /etc/nginx/sites-available /root/.auth_details 2>/dev/null
    fi

    # Retention (Smart Rotation: 7 days)
    find "$BACKUP_DIR" -name "db_*.sql" -type f -mtime +7 -delete 2>/dev/null || true
    find "$BACKUP_DIR" -name "files_*.tar.gz" -type f -mtime +7 -delete 2>/dev/null || true

    success "Backup Complete: $TIMESTAMP"
}

manage_restore() {
    clear >&3 || true
    msg "SYSTEM RESTORE WIZARD"
    warn "This will OVERWRITE current data. Recommended to backup first."

    # List Backups
    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A "$BACKUP_DIR")" ]; then
        error "No backups found in $BACKUP_DIR"
        ask "Press Enter to return..." dummy
        return
    fi

    echo "Available File Backups:"
    local i=1
    local backups=()
    while IFS= read -r file; do
        echo "$i) $(basename "$file")"
        backups+=("$file")
        ((i++))
    done < <(find "$BACKUP_DIR" -name "files_*.tar.gz" | sort -r)

    ask "Select backup to restore (Number):" choice
    if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le "${#backups[@]}" ]; then
        local selected_file="${backups[$((choice-1))]}"
        local timestamp=$(basename "$selected_file" | sed 's/files_//;s/.tar.gz//')
        local db_file="$BACKUP_DIR/db_$timestamp.sql"

        ask "Are you sure you want to restore from $timestamp? (y/n):" confirm
        if [[ "$confirm" == "y" ]]; then
            msg "Restoring Files..."
            tar -xf "$selected_file" -C /

            if [ -f "$db_file" ]; then
                msg "Restoring Database..."
                if [ -z "${DB_ROOT_PASS:-}" ]; then
                     DB_ROOT_PASS=$(get_auth_value "mysql_root_password")
                fi
                if [ -n "$DB_ROOT_PASS" ]; then
                    mysql -u root --password="$DB_ROOT_PASS" < "$db_file"
                    success "Database Restored"
                else
                    warn "DB Password missing, skipping DB restore."
                fi
            else
                warn "Matching DB backup ($db_file) not found."
            fi

            msg "Restarting Services..."
            systemctl restart nginx || true
            systemctl restart mariadb || true

            # Restore Docker Containers (Recreate them)
            # Iterate through opt directories and run docker compose up
            if [ -d "/opt" ]; then
                for d in /opt/*; do
                    if [ -f "$d/docker-compose.yml" ]; then
                        msg "Restoring container $(basename "$d")..."
                        (cd "$d" && docker compose up -d) || warn "Failed to start $(basename "$d")"
                    fi
                done
            fi

            success "Restore Complete!"
        else
            msg "Restore Cancelled."
        fi
    else
        error "Invalid selection."
    fi
    ask "Press Enter to continue..." dummy
}

health_check() {
    clear >&3 || true
    msg "SYSTEM HEALTH CHECK"
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    echo -e "SERVICE              | STATUS      | URL" >&3
    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3

    check_url() {
        local name=$1
        local url=$2
        local code=$(curl -s -o /dev/null -w "%{http_code}" "$url" || echo "ERR")
        if [[ "$code" == "200" || "$code" == "302" ]]; then
            echo -e "${name} | ${GREEN}ONLINE ($code)${NC} | $url" >&3
        else
            echo -e "${name} | ${RED}DOWN ($code)${NC}   | $url" >&3
        fi
    }

    # System Services
    if systemctl is-active --quiet nginx; then
        echo -e "Nginx                | ${GREEN}RUNNING${NC}     | -" >&3
    else
        echo -e "Nginx                | ${RED}STOPPED${NC}     | -" >&3
    fi

    if systemctl is-active --quiet mariadb; then
        echo -e "MariaDB              | ${GREEN}RUNNING${NC}     | -" >&3
    else
        echo -e "MariaDB              | ${RED}STOPPED${NC}     | -" >&3
    fi

    # Web Services
    if [ -d "/opt/gitea" ]; then check_url "Gitea               " "https://git.$DOMAIN"; fi
    if [ -d "/opt/nextcloud" ]; then check_url "Nextcloud           " "https://cloud.$DOMAIN"; fi
    if [ -d "/opt/portainer" ] || docker ps | grep -q portainer; then check_url "Portainer           " "https://admin.$DOMAIN"; fi # Assuming admin or direct
    if [ -d "/opt/uptimekuma" ]; then check_url "Uptime Kuma         " "https://status.$DOMAIN"; fi
    if [ -d "/opt/netdata" ] || docker ps | grep -q netdata; then check_url "Netdata             " "https://netdata.$DOMAIN"; fi

    echo -e "${YELLOW}-----------------------------------------------------${NC}" >&3
    ask "Press Enter to continue..." dummy
}

detect_profile() {
    local profile_file="${PROFILE_FILE:-/etc/cyl_profile}"
    TOTAL_RAM=$(free -m | awk '/Mem:/ { print $2 }')
    msg "Detected RAM: ${TOTAL_RAM}MB"

    if [ "$TOTAL_RAM" -lt 3800 ]; then
        PROFILE="LOW"
        msg "Profile selected: LOW RESOURCE (Optimization for stability)"
        # Install ZRAM for low profile
        if ! command -v zramctl >/dev/null; then
            msg "Installing ZRAM for memory compression..."
            apt-get install -y zram-tools >/dev/null
        fi
        echo "LOW" > "$profile_file"
    else
        PROFILE="HIGH"
        msg "Profile selected: HIGH PERFORMANCE (Optimization for speed)"
        echo "HIGH" > "$profile_file"
    fi
}

tune_system() {
    msg "Applying System Tuning (Sheldon Cooper Mode)..."
    PROFILE=$(cat /etc/cyl_profile 2>/dev/null || echo "HIGH")

    # 0. Kernel & TCP Tuning
    # Optimize for high concurrency and low latency
    # We use a dedicated file for idempotency
    cat <<EOF > /etc/sysctl.d/99-cylae-tuning.conf
# Network Tuning
net.core.somaxconn = 4096
net.core.netdev_max_backlog = 16384
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15

# Memory Tuning
vm.swappiness = 10
vm.vfs_cache_pressure = 50
EOF
    sysctl --system >/dev/null

    # 1. Database Tuning (MariaDB)
    # Check if MariaDB is installed first
    if [ -d "/etc/mysql/mariadb.conf.d" ]; then
        if [ "$PROFILE" == "LOW" ]; then
            cat <<EOF > /etc/mysql/mariadb.conf.d/99-tuning.cnf
[mysqld]
innodb_buffer_pool_size = 128M
max_connections = 50
performance_schema = OFF
bind-address = 0.0.0.0
EOF
        else
            cat <<EOF > /etc/mysql/mariadb.conf.d/99-tuning.cnf
[mysqld]
innodb_buffer_pool_size = 1G
max_connections = 200
bind-address = 0.0.0.0
EOF
        fi
        systemctl restart mariadb || warn "Failed to restart MariaDB"
    fi

    # 2. PHP Tuning
    if command -v php >/dev/null; then
        PHP_VER=$(php -r "echo PHP_MAJOR_VERSION.'.'.PHP_MINOR_VERSION;")
        PHP_FPM_CONF="/etc/php/$PHP_VER/fpm/pool.d/www.conf"
        PHP_INI="/etc/php/$PHP_VER/fpm/php.ini"

        if [ -f "$PHP_FPM_CONF" ]; then
            if [ "$PROFILE" == "LOW" ]; then
                sed -i 's/^pm.max_children = .*/pm.max_children = 10/' "$PHP_FPM_CONF"
                sed -i 's/^memory_limit = .*/memory_limit = 256M/' "$PHP_INI"
            else
                sed -i 's/^pm.max_children = .*/pm.max_children = 50/' "$PHP_FPM_CONF"
                sed -i 's/^memory_limit = .*/memory_limit = 512M/' "$PHP_INI"
            fi
            # Common
            sed -i 's/^upload_max_filesize = .*/upload_max_filesize = 512M/' "$PHP_INI"
            sed -i 's/^post_max_size = .*/post_max_size = 512M/' "$PHP_INI"

            # Opcache Tuning
            sed -i 's/;opcache.enable=1/opcache.enable=1/' "$PHP_INI"
            sed -i 's/;opcache.memory_consumption=.*/opcache.memory_consumption=128/' "$PHP_INI"
            sed -i 's/;opcache.interned_strings_buffer=.*/opcache.interned_strings_buffer=8/' "$PHP_INI"
            sed -i 's/;opcache.max_accelerated_files=.*/opcache.max_accelerated_files=10000/' "$PHP_INI"
            sed -i 's/;opcache.validate_timestamps=.*/opcache.validate_timestamps=0/' "$PHP_INI"

            systemctl restart "php$PHP_VER-fpm" || warn "Failed to restart PHP-FPM"
        fi
    fi

    success "System Tuned for $PROFILE usage."
}

system_update() {
    msg "Updating System..."
    apt-get update && apt-get upgrade -y >/dev/null

    msg "Updating Docker Images..."
    if docker ps >/dev/null 2>&1; then
        # Bolt: Parallelize image pulls to speed up updates
        # Limit concurrency based on profile to avoid killing low-end servers
        local profile=$(cat /etc/cyl_profile 2>/dev/null || echo "HIGH")
        local jobs=4
        if [ "$profile" == "LOW" ]; then
             jobs=2
        fi

        docker images --format "{{.Repository}}:{{.Tag}}" | grep -v "<none>" | xargs -P "$jobs" -n 1 docker pull 2>/dev/null || true
        docker image prune -f >/dev/null
    fi
    success "System Updated"
}

setup_autoupdate() {
    msg "Configuring Auto-Update..."
    # Ensure current directory is correct
    local install_dir="${INSTALL_DIR:-$(pwd)}"
    local target_lib_dir="/usr/local/lib/cyl_manager"

    # Create library directory
    mkdir -p "$target_lib_dir"

    # Copy source files
    if [ -d "$install_dir/src" ]; then
        cp -r "$install_dir/src" "$target_lib_dir/"
    else
        warn "Source directory src/ not found in $install_dir"
    fi

    # Update main script wrapper
    # We modify the install.sh being copied to point to the library dir
    # Or we can just symlink

    cat <<EOF > /usr/local/bin/server_manager.sh
#!/bin/bash
INSTALL_DIR="$install_dir"
cd "\$INSTALL_DIR"
exec ./install.sh
EOF
    chmod +x /usr/local/bin/server_manager.sh

    if [ -f "$install_dir/auto_update.sh" ]; then
        cp "$install_dir/auto_update.sh" /usr/local/bin/server_autoupdate.sh
        chmod +x /usr/local/bin/server_autoupdate.sh

        CRON_CMD="0 4 * * * /usr/local/bin/server_autoupdate.sh"
        (crontab -l 2>/dev/null | grep -v "server_autoupdate.sh" || true; echo "$CRON_CMD") | crontab -
        success "Auto-Update Scheduled (Daily @ 04:00)"
    else
        warn "auto_update.sh not found, skipping cron setup."
    fi
}

setup_watchdog() {
    msg "Configuring Self-Healing Watchdog..."
    cat <<'EOF' > /usr/local/bin/cyl_watchdog.sh
#!/bin/bash
# Check critical services
if ! systemctl is-active --quiet nginx; then
    echo "$(date): Nginx down. Restarting..." >> /var/log/cyl_watchdog.log
    systemctl restart nginx
fi

if ! systemctl is-active --quiet mariadb; then
    echo "$(date): MariaDB down. Restarting..." >> /var/log/cyl_watchdog.log
    systemctl restart mariadb
fi

# Clean zombie Docker processes if any
# (Optional advanced logic could go here)
EOF
    chmod +x /usr/local/bin/cyl_watchdog.sh

    CRON_JOB="0 * * * * /usr/local/bin/cyl_watchdog.sh"
    (crontab -l 2>/dev/null | grep -v "cyl_watchdog.sh" || true; echo "$CRON_JOB") | crontab -
    success "Watchdog Enabled (Hourly)"
}
