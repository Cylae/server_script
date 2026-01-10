#!/bin/bash
set -euo pipefail

# ==============================================================================
# UTILS MODULE
# General utilities (Backup, System Tuning, etc)
# ==============================================================================

manage_backup() {
    msg "Starting Backup..."
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)

    # DB Dump
    if command -v mysqldump >/dev/null; then
        mysqldump -u root --password="$DB_ROOT_PASS" --all-databases > "$BACKUP_DIR/db_$TIMESTAMP.sql"
    fi

    # Files
    tar -czf "$BACKUP_DIR/files_$TIMESTAMP.tar.gz" /opt /var/www /etc/nginx/sites-available /root/.auth_details 2>/dev/null

    # Retention (keep last 5)
    ls -tp "$BACKUP_DIR"/db_*.sql 2>/dev/null | grep -v '/$' | tail -n +6 | xargs -I {} rm -- {} 2>/dev/null || true
    ls -tp "$BACKUP_DIR"/files_*.tar.gz 2>/dev/null | grep -v '/$' | tail -n +6 | xargs -I {} rm -- {} 2>/dev/null || true

    success "Backup Complete: $TIMESTAMP"
}

detect_profile() {
    TOTAL_RAM=$(free -m | awk '/Mem:/ { print $2 }')
    msg "Detected RAM: ${TOTAL_RAM}MB"

    if [ "$TOTAL_RAM" -lt 3800 ]; then
        PROFILE="LOW"
        msg "Profile selected: LOW RESOURCE (Optimization for stability)"
        echo "LOW" > /etc/cyl_profile
    else
        PROFILE="HIGH"
        msg "Profile selected: HIGH PERFORMANCE (Optimization for speed)"
        echo "HIGH" > /etc/cyl_profile
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
        docker images --format "{{.Repository}}:{{.Tag}}" | grep -v "<none>" | xargs -P 4 -n 1 docker pull 2>/dev/null || true
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
