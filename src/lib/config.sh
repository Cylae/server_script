#!/bin/bash
set -euo pipefail

# ==============================================================================
# CONFIG MODULE
# Configuration Loading, Saving, and Secrets Management
# ==============================================================================

CONFIG_FILE="/etc/cyl_manager.conf"
AUTH_FILE="${AUTH_FILE:-/root/.auth_details}"
BACKUP_DIR="/var/backups/cyl_manager"

load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
    fi

    # Interactive Setup if missing domain
    if [ -z "${DOMAIN:-}" ]; then
        clear >&3 || true
        echo -e "${PURPLE}=================================================================${NC}" >&3
        echo -e "${PURPLE}  CYL.AE SERVER MANAGER - INITIAL SETUP${NC}" >&3
        echo -e "${PURPLE}=================================================================${NC}" >&3
        echo -e "Welcome! Let's configure your server." >&3
        echo "" >&3

        while [ -z "${DOMAIN:-}" ]; do
            ask "Enter your domain name (e.g., example.com):" INPUT_DOMAIN
            if [[ "$INPUT_DOMAIN" =~ ^[a-zA-Z0-9]+([-.][a-zA-Z0-9]+)*\.[a-zA-Z]{2,}$ ]]; then
                DOMAIN="$INPUT_DOMAIN"
            else
                echo "Error: Invalid domain format." >&3
            fi
        done

        # Save to config
        echo "DOMAIN=\"$DOMAIN\"" >> "$CONFIG_FILE"
        echo "EMAIL=\"admin@$DOMAIN\"" >> "$CONFIG_FILE"
        echo "INSTALL_DIR=\"$(pwd)\"" >> "$CONFIG_FILE"
        chmod 600 "$CONFIG_FILE"
    fi

    # Do not overwrite if already set
    if [ -z "${EMAIL:-}" ]; then
        EMAIL="admin@$DOMAIN"
    fi
}

validate_password() {
    local pass="$1"
    # Basic complexity: length >= 8
    if [ ${#pass} -lt 8 ]; then
        warn "Password is too short (min 8 chars)."
        return 1
    fi
    return 0
}

generate_password() {
    openssl rand -base64 16 | tr -dc 'a-zA-Z0-9' | head -c 24
}

get_auth_value() {
    local key="$1"
    if [ -f "$AUTH_FILE" ]; then
        grep "^${key}=" "$AUTH_FILE" 2>/dev/null | cut -d= -f2- | tail -n 1 || true
    fi
}

save_credential() {
    local key="$1"
    local value="$2"

    # Ensure file exists and is secure
    if [ ! -f "$AUTH_FILE" ]; then
        touch "$AUTH_FILE"
        chmod 600 "$AUTH_FILE"
    fi

    # Update or Append credential
    # We remove the old line first to avoid sed delimiter issues with special chars in value
    if grep -q "^${key}=" "$AUTH_FILE"; then
        sed -i "/^${key}=/d" "$AUTH_FILE"
    fi
    echo "${key}=${value}" >> "$AUTH_FILE"
}

get_or_create_password() {
    local key="$1"
    local saved=$(get_auth_value "$key")
    if [ -n "$saved" ]; then
        echo "$saved"
    else
        local new_pass=$(generate_password)
        save_credential "$key" "$new_pass"
        echo "$new_pass"
    fi
}
