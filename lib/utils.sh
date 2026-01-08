#!/bin/bash
# lib/utils.sh - Utility functions for CYL Server Manager

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging
log() { echo -e "$(date +'%Y-%m-%d %H:%M:%S') - $1"; }
msg() { echo -e "${CYAN}➜ $1${NC}" >&3; log "INFO: $1"; }
success() { echo -e "${GREEN}✔ $1${NC}" >&3; log "SUCCESS: $1"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}" >&3; log "WARN: $1"; }
error() { echo -e "${RED}✖ $1${NC}" >&3; log "ERROR: $1"; }
fatal() { error "$1"; exit 1; }

# Helper for prompts to ensure they go to fd 3 (user) not log
ask() {
    local prompt="$1"
    local var_name="$2"
    echo -ne "${YELLOW}$prompt${NC} " >&3
    read -r "$var_name"
}

check_root() {
    [[ $EUID -ne 0 ]] && fatal "This script must be run as root."
}

check_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" != "debian" && "$ID" != "ubuntu" ]]; then
             warn "Detected OS: $ID. This script is optimized for Debian/Ubuntu."
             ask "Continue anyway? (y/n):" confirm
             if [[ "$confirm" != "y" ]]; then exit 1; fi
        fi
    else
        warn "Cannot detect OS. Proceed with caution."
    fi
}

load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
    fi

    # Interactive Setup if missing domain
    if [ -z "${DOMAIN:-}" ]; then
        clear >&3
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

    EMAIL="admin@$DOMAIN"
}

generate_password() {
    openssl rand -base64 16 | tr -dc 'a-zA-Z0-9' | head -c 16
}

get_auth_value() {
    local key="$1"
    if [ -f "$AUTH_FILE" ]; then
        grep "^${key}=" "$AUTH_FILE" | cut -d= -f2- | tail -n 1
    fi
}

get_or_create_password() {
    local key="$1"
    local saved=$(get_auth_value "$key")
    if [ -n "$saved" ]; then
        echo "$saved"
    else
        local new_pass=$(generate_password)
        echo "${key}=${new_pass}" >> "$AUTH_FILE"
        echo "$new_pass"
    fi
}

# --- New Features ---

# ask_credential: Prompts for credentials, defaults to auto-generated
# Usage: ask_credential "Service Name" "default_user_prefix" "auth_key_prefix"
# Returns: SET_USER and SET_PASS variables
ask_credential() {
    local service_name="$1"
    local default_user_prefix="$2"
    local auth_key_prefix="$3"

    msg "Configuring Credentials for $service_name..."

    # Username
    local default_user="${default_user_prefix}"
    if [[ "$default_user_prefix" != *@* ]]; then
        # If no domain part, assume just user part, but some services might want full email
        : # Do nothing
    fi

    # Try to fetch existing
    local existing_user=$(get_auth_value "${auth_key_prefix}_user")
    if [ -n "$existing_user" ]; then
        default_user="$existing_user"
    fi

    ask "Enter Username [$default_user]:" input_user
    if [ -z "$input_user" ]; then
        SET_USER="$default_user"
    else
        SET_USER="$input_user"
    fi

    # Password
    local existing_pass=$(get_auth_value "${auth_key_prefix}_pass")
    local prompt_pass="[Auto-Generate]"
    if [ -n "$existing_pass" ]; then
        prompt_pass="[Keep Existing]"
    fi

    ask "Enter Password $prompt_pass:" input_pass
    if [ -z "$input_pass" ]; then
        if [ -n "$existing_pass" ]; then
            SET_PASS="$existing_pass"
        else
            SET_PASS=$(generate_password)
        fi
    else
        SET_PASS="$input_pass"
    fi

    # Save if changed or new
    if [ "$SET_USER" != "$existing_user" ]; then
        echo "${auth_key_prefix}_user=${SET_USER}" >> "$AUTH_FILE"
    fi
    # If password is new (not matching existing), save it.
    # Or if we just generated it.
    if [ "$SET_PASS" != "$existing_pass" ]; then
        echo "${auth_key_prefix}_pass=${SET_PASS}" >> "$AUTH_FILE"
    fi
}

show_service_report() {
    local service_name="$1"
    local url="$2"
    local user="$3"
    local pass="$4"

    echo -e "" >&3
    echo -e "${GREEN}=====================================================${NC}" >&3
    echo -e "${GREEN}  INSTALLATION SUCCESSFUL: $service_name${NC}" >&3
    echo -e "${GREEN}=====================================================${NC}" >&3
    echo -e "  URL      : ${CYAN}$url${NC}" >&3
    echo -e "  Username : ${CYAN}$user${NC}" >&3
    echo -e "  Password : ${CYAN}$pass${NC}" >&3
    echo -e "${GREEN}=====================================================${NC}" >&3
    echo -e "" >&3
    ask "Press Enter to continue..." dummy
}
