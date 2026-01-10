#!/bin/bash
# ==============================================================================
#  CYL.AE SERVER MANAGER V8.0 (Secure Edition)
#  Production-grade, hardened, and modular deployment suite.
# ==============================================================================

# Strict mode: exit on error, undefined vars, or pipe failures.
set -u
set -o pipefail
set -e

# Get the directory of the script
INSTALL_DIR="$(dirname "$(realpath "$0")")"
cd "$INSTALL_DIR"

# Redirect all output to log file, but keep fd 3 for user interaction
LOG_FILE="/var/log/server_manager.log"
exec 3>&1 1>>"$LOG_FILE" 2>&1

# Error handling
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "\n\033[0;31m✖ Script failed with exit code $exit_code.\033[0m" >&3
        echo -e "\033[0;33mLast 15 lines of $LOG_FILE:\033[0m" >&3
        echo "--------------------------------------------------" >&3
        tail -n 15 "$LOG_FILE" >&3
        echo "--------------------------------------------------" >&3
        echo -e "\nPlease check the log file for more details." >&3
    fi
}
trap cleanup EXIT

# Source the main application logic
if [ -f "src/main.sh" ]; then
    source src/main.sh
    run_main
else
    echo "Error: src/main.sh not found. Please ensure you are running this from the repo root." >&3
    exit 1
fi
