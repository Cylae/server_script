#!/bin/bash
set -euo pipefail

# ==============================================================================
# CYLAE SERVER MANAGER - BOOTSTRAP
# ==============================================================================

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    # Pass directly to python if installed, else show basic help
    if command -v python3 >/dev/null 2>&1; then
        INSTALL_DIR="$(dirname "$(realpath "$0")")"
        export PYTHONPATH="$INSTALL_DIR/src"
        exec python3 -m cylae.cli --help
    else
        echo "Usage: ./install.sh [OPTIONS]"
        echo "  -h, --help    Show help"
        exit 0
    fi
fi

if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges."
    if command -v sudo >/dev/null 2>&1; then
        echo "Escalating privileges..."
        exec sudo "$0" "$@"
    else
        echo "Error: Please run as root."
        exit 1
    fi
fi

echo "Bootstrapping Cylae Server Manager (Python Edition)..."

# Install Python3 if missing
if ! command -v python3 >/dev/null 2>&1; then
    echo "Installing Python3..."
    apt-get update -q
    apt-get install -y python3
fi

# We might need pip later, but currently standard library only.

# Detect directory
INSTALL_DIR="$(dirname "$(realpath "$0")")"
cd "$INSTALL_DIR"

# Set PYTHONPATH to include src
export PYTHONPATH="$INSTALL_DIR/src"

# Run the Python CLI
exec python3 -m cylae.cli "$@"
