#!/bin/bash
# ==============================================================================
#  CYL.AE SERVER MANAGER V8.1 (Robust Edition)
#  Production-grade, hardened, and modular deployment suite.
# ==============================================================================

# Strict mode: exit on error, undefined vars, or pipe failures.
set -u
set -o pipefail
set -e

# ------------------------------------------------------------------------------
# PRE-FLIGHT CHECKS & ROOT ESCALATION
# ------------------------------------------------------------------------------

# 1. Root Check & Escalation
if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges."
    if command -v sudo >/dev/null 2>&1; then
        echo "Escalating privileges..."
        exec sudo "$0" "$@"
    else
        echo "Error: 'sudo' is not installed and you are not root."
        echo "Please run as root or install sudo."
        exit 1
    fi
fi

# 2. Setup Logging (Safe now that we are root)
LOG_FILE="/var/log/server_manager.log"
if [ ! -f "$LOG_FILE" ]; then
    touch "$LOG_FILE"
    chmod 600 "$LOG_FILE"
fi

# Redirect stdout/stderr to log, keep fd 3 for user interaction
exec 3>&1 1>>"$LOG_FILE" 2>&1

# 3. Error Handling Trap
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "\n\033[0;31m✖ Script failed with exit code $exit_code.\033[0m" >&3
        echo -e "\033[0;33mLast 20 lines of $LOG_FILE:\033[0m" >&3
        echo "--------------------------------------------------" >&3
        tail -n 20 "$LOG_FILE" >&3
        echo "--------------------------------------------------" >&3
        echo -e "\nPlease check the log file for more details." >&3
    fi
}
trap cleanup EXIT

# 4. Disk Space Check
check_disk_space() {
    local required_mb=5120 # 5GB
    # Check root partition available space in MB
    local available_mb
    if command -v df >/dev/null 2>&1; then
         available_mb=$(df / . --output=avail -B 1M | tail -n 1)
         if [ "$available_mb" -lt "$required_mb" ]; then
            echo "Error: Insufficient disk space." >&3
            echo "Required: ${required_mb}MB, Available: ${available_mb}MB." >&3
            exit 1
         fi
    fi
}
check_disk_space

# ------------------------------------------------------------------------------
# DEPENDENCY CHECK & INSTALLATION
# ------------------------------------------------------------------------------
echo "Checking core dependencies..." >&3

install_if_missing() {
    local cmd=$1
    local pkg=${2:-$1} # Default to cmd name if pkg name not provided

    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Installing missing dependency: $pkg..." >&3
        export DEBIAN_FRONTEND=noninteractive

        # Simple retry logic for apt
        local retries=5
        local count=0
        until apt-get update -q && apt-get install -y "$pkg"; do
            exit_code=$?
            count=$((count + 1))
            if [ $count -ge $retries ]; then
                echo "Error: Failed to install $pkg after $retries attempts." >&3
                return 1
            fi
            echo "Apt failed. Retrying in 5s..." >&3
            sleep 5
        done
    fi
}

install_if_missing curl
install_if_missing wget
install_if_missing git
install_if_missing jq
install_if_missing lsb_release lsb-release

# ------------------------------------------------------------------------------
# EXECUTION
# ------------------------------------------------------------------------------

# Get directory safely
if command -v realpath >/dev/null 2>&1; then
    INSTALL_DIR="$(dirname "$(realpath "$0")")"
else
    INSTALL_DIR="$(cd "$(dirname "$0")" && pwd)"
fi
cd "$INSTALL_DIR"

if [ -f "src/main.sh" ]; then
    source src/main.sh
    run_main
else
    echo "Error: src/main.sh not found in $INSTALL_DIR." >&3
    echo "Please ensure you have cloned the full repository." >&3
    exit 1
fi
