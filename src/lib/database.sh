#!/bin/bash

# ==============================================================================
# DATABASE MODULE
# Database helpers
# ==============================================================================

init_db_password() {
    if grep -q "mysql_root_password" $AUTH_FILE; then
        DB_ROOT_PASS=$(get_auth_value "mysql_root_password")
    else
        if [ -f /root/.mariadb_auth ]; then
             DB_ROOT_PASS=$(grep mysql_root_password /root/.mariadb_auth | cut -d= -f2)
        else
             DB_ROOT_PASS=$(generate_password)
             save_credential "mysql_root_password" "$DB_ROOT_PASS"
             mysql -e "ALTER USER 'root'@'localhost' IDENTIFIED BY '$DB_ROOT_PASS'; FLUSH PRIVILEGES;" || true
        fi
    fi
    export DB_ROOT_PASS
}

ensure_db() {
    # Usage: ensure_db dbname username password
    local db=$1
    local user=$2
    local pass=$3
    # Use ALTER USER to ensure password consistency on reinstall
    mysql -u root --password="$DB_ROOT_PASS" -e "CREATE DATABASE IF NOT EXISTS \`$db\`; CREATE USER IF NOT EXISTS '$user'@'%' IDENTIFIED BY '$pass'; GRANT ALL PRIVILEGES ON \`$db\`.* TO '$user'@'%'; FLUSH PRIVILEGES; ALTER USER '$user'@'%' IDENTIFIED BY '$pass';"
}
