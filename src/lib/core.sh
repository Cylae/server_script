#!/bin/bash
set -euo pipefail

# ==============================================================================
# CORE MODULE
# Logging, Colors, Error Handling, and Basic Checks
# ==============================================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
# We assume LOG_FILE is set by the configuration module or main script.
# If not, we default to /dev/null for the file output.

log() {
    local timestamp
    timestamp=$(date +'%Y-%m-%d %H:%M:%S')
    local log_file="${LOG_FILE:-/dev/null}"
    echo -e "$timestamp - $1" >> "$log_file"
}

msg() {
    echo -e "${CYAN}➜ $1${NC}" >&3
    log "INFO: $1"
}

success() {
    echo -e "${GREEN}✔ $1${NC}" >&3
    log "SUCCESS: $1"
}

warn() {
    echo -e "${YELLOW}⚠ $1${NC}" >&3
    log "WARN: $1"
}

error() {
    echo -e "${RED}✖ $1${NC}" >&3
    log "ERROR: $1"
}

fatal() {
    error "$1"
    exit 1
}

# Helper for prompts to ensure they go to fd 3 (user) not log
ask() {
    local prompt="$1"
    local var_name="$2"
    echo -ne "${YELLOW}$prompt${NC} " >&3
    read -r "$var_name"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        fatal "This script must be run as root."
    fi
}

check_os() {
    if [ -f /etc/os-release ]; then
        # Use grep and cut to avoid dangerous eval and potential pipefail issues
        local ID=$(grep "^ID=" /etc/os-release | cut -d= -f2 | tr -d '"') || true
        if [[ "${ID:-}" != "debian" && "${ID:-}" != "ubuntu" ]]; then
             warn "Detected OS: ${ID:-unknown}. This script is optimized for Debian/Ubuntu."
             ask "Continue anyway? (y/n):" confirm
             if [[ "$confirm" != "y" ]]; then exit 1; fi
        fi
    else
        warn "Cannot detect OS. Proceed with caution."
    fi
}
