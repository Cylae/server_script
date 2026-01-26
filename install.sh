#!/bin/bash
set -e

# "One Click" Server Manager Installer
# Designed for fresh Debian/Ubuntu VMs (GCP, AWS, DigitalOcean, etc.)

# 1. Ensure Root Privileges
if [ "$EUID" -ne 0 ]; then
  echo "Elevating privileges... (sudo)"

  # Check if running via pipe/stdin
  if [ -t 0 ]; then
    # Interactive execution: re-run self with sudo
    sudo "$0" "$@"
    exit $?
  else
    # Piped execution: download script to temp file and run with sudo
    TEMP_SCRIPT=$(mktemp)
    # We can't access $0 content easily if piped, so we re-download or copy logic.
    # But since we are likely running 'curl ... | bash', $0 is bash.
    # To support this robustly, we should assume the user is piping from the URL.
    # We will re-curl the script.

    echo "Downloading installer to temporary file for elevation..."
    curl -sL https://raw.githubusercontent.com/Cylae/server_script/main/install.sh -o "$TEMP_SCRIPT"
    chmod +x "$TEMP_SCRIPT"
    sudo "$TEMP_SCRIPT" "$@"
    rm -f "$TEMP_SCRIPT"
    exit $?
  fi
fi

echo "=== Server Manager Bootstrap ==="
echo "Starting installation on $(hostname)..."

# 2. Basic Dependencies (for bootstrapping)
# We need git to clone the repo (if running via curl) and build-essential for Rust compilation
echo "Installing bootstrap dependencies..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq git build-essential curl libssl-dev pkg-config

# 3. Clone Repository (if not already local)
# If the script is run from inside the repo, we use the current dir.
# If run via curl, we clone to /opt/server_manager_source
if [ -d "server_manager" ] && [ -f "server_manager/Cargo.toml" ]; then
    echo "Running from source directory..."
    SRC_DIR=$(pwd)
else
    echo "Cloning repository..."
    SRC_DIR="/opt/server_manager_source"
    if [ -d "$SRC_DIR" ]; then
        rm -rf "$SRC_DIR"
    fi
    git clone https://github.com/Cylae/server_script.git "$SRC_DIR"
fi

cd "$SRC_DIR/server_manager"

# 4. Install Rust (if missing)
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 5. Build and Run Server Manager
echo "Building Server Manager..."
cargo build --release

echo "Launching Server Manager Installer..."
./target/release/server_manager install
