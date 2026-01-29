#!/bin/bash
set -euo pipefail

# Log file
LOG_FILE="/var/log/my-server-install.log"

# Defensive: Trap errors
trap 'error_handler $? $LINENO $BASH_COMMAND' ERR

error_handler() {
    local exit_code=$1
    local line_no=$2
    local command=$3
    echo "Error: Command \"$command\" failed with exit code $exit_code on line $line_no."
    exit "$exit_code"
}

# Redirect output to log, showing user info on fd 3
exec 3>&1
exec > "$LOG_FILE" 2>&1

info() {
    echo "$@" >&3
    echo "$@"
}

info "Starting Server Setup... Logs at $LOG_FILE"

# 1. Root Check
if [ "$EUID" -ne 0 ]; then
    info "Error: This script must be run as root."
    exit 1
fi

# 2. Non-Interactive Mode
export DEBIAN_FRONTEND=noninteractive

# 3. Network Check
info "Checking connectivity..."
if ! ping -c 1 google.com &> /dev/null; then
    info "Error: No internet connection."
    exit 1
fi

# 4. Dependency Bootstrap
info "Bootstrapping dependencies..."
MISSING_DEPS=""
# Added psmisc for fuser command used in lock check
for dep in curl wget git sudo jq bc fuser; do
    if ! command -v "$dep" &> /dev/null; then
        # Map command to package if necessary (fuser -> psmisc)
        if [ "$dep" == "fuser" ]; then
            MISSING_DEPS="$MISSING_DEPS psmisc"
        else
            MISSING_DEPS="$MISSING_DEPS $dep"
        fi
    fi
done

if [ -n "$MISSING_DEPS" ]; then
    info "Installing missing dependencies: $MISSING_DEPS"

    # Robust install loop to handle boot-time locks (e.g. unattended-upgrades)
    # We can't use fuser yet because we haven't installed psmisc.
    for i in {1..20}; do
        # We redirect stderr to null to avoid scary messages during lock contention,
        # but if it fails genuinely we might miss info.
        # However, we rely on the loop.
        if apt-get update -qq 2>/dev/null && apt-get install -y -qq $MISSING_DEPS 2>/dev/null; then
            break
        fi
        info "Apt lock busy or network error. Retrying in 10s... ($i/20)"
        sleep 10
        if [ "$i" -eq 20 ]; then
             info "Failed to bootstrap dependencies. Apt is persistently locked or network is down."
             # Try one last time with output visible
             apt-get update -qq && apt-get install -y -qq $MISSING_DEPS
             exit 1
        fi
    done
fi

# 5. System Update
info "Updating system packages..."
# Handle lock file
wait_for_apt_lock() {
    local locks="/var/lib/dpkg/lock /var/lib/apt/lists/lock /var/lib/dpkg/lock-frontend"
    for lock in $locks; do
        while fuser "$lock" >/dev/null 2>&1; do
            info "Waiting for apt lock on $lock..."
            sleep 5
        done
    done
}

wait_for_apt_lock
apt-get update -qq
apt-get upgrade -y -qq
info "System updated."

# --- User Management ---

MANAGED_GROUP="server_users"
CREDENTIALS_FILE="/root/credentials.txt"

setup_user_management() {
    info "Configuring User Management..."

    # Create group
    if ! getent group "$MANAGED_GROUP" >/dev/null; then
        groupadd "$MANAGED_GROUP"
        info "Created system group: $MANAGED_GROUP"
    fi

    # Ensure credentials file exists and is secure
    touch "$CREDENTIALS_FILE"
    chmod 600 "$CREDENTIALS_FILE"
}

create_managed_user() {
    local username=$1
    local password=${2:-}

    if id "$username" &>/dev/null; then
        info "User $username already exists. Skipping creation."
        return 0
    fi

    # Generate password if not provided
    if [ -z "$password" ]; then
        # Robust password generation using urandom
        password=$(tr -dc 'A-Za-z0-9!@#%^&*' < /dev/urandom | head -c 16)
    fi

    info "Creating user: $username"
    useradd -m -s /bin/bash -g "$MANAGED_GROUP" "$username"
    echo "$username:$password" | chpasswd

    # Secure home directory (chmod 750)
    chmod 750 "/home/$username"

    # Save credentials
    echo "User: $username | Password: $password | Date: $(date)" >> "$CREDENTIALS_FILE"
    info "Credentials for $username saved to $CREDENTIALS_FILE"
}

# --- Security ---

setup_security() {
    info "Configuring Security (UFW & Fail2Ban)..."

    # Install UFW and Fail2Ban
    apt-get install -y -qq ufw fail2ban

    # Configure UFW
    ufw default deny incoming
    ufw default allow outgoing

    # Allow SSH (Port 22).
    ufw allow 22/tcp
    # Allow HTTP/HTTPS
    ufw allow 80/tcp
    ufw allow 443/tcp

    # Enable UFW non-interactively
    ufw --force enable
    info "Firewall enabled (SSH, HTTP, HTTPS allowed)."

    # Configure Fail2Ban
    systemctl enable fail2ban
    systemctl restart fail2ban
    info "Fail2Ban enabled."
}

# --- Storage Quota ---

enable_quotas() {
    info "Configuring Disk Quotas..."

    # Install quota tool
    apt-get install -y -qq quota

    # Detect the partition for /home
    # If /home is a mount point, use it. Otherwise /
    local target_mount="/"
    if mountpoint -q /home; then
        target_mount="/home"
    fi

    local fstab_file="/etc/fstab"

    info "Detected mount point for quotas: $target_mount"

    # Check if quotas are already enabled in fstab
    if grep -q "$target_mount.*usrquota" "$fstab_file"; then
        info "Quotas appear to be configured in fstab."
    else
        info "Enabling quotas in $fstab_file..."
        cp "$fstab_file" "${fstab_file}.bak"

        # Use awk to safely append options
        awk -v mount="$target_mount" '
        $2 == mount {
            if ($4 !~ /usrquota/ && $4 !~ /grpquota/) {
                $4 = $4 ",usrquota,grpquota"
            }
        }
        { print }
        ' "${fstab_file}.bak" > "$fstab_file"

        info "Updated fstab. Remounting $target_mount..."
        if ! mount -o remount "$target_mount"; then
             info "Remount failed. A reboot may be required to enable quotas."
        fi
    fi

    # Initialize quotas
    info "Initializing quotas (this may take a moment)..."
    # quotacheck: -u user, -g group, -m no-remount (force), -c create files
    # We use -ugm.
    quotacheck -ugm "$target_mount" >/dev/null 2>&1 || true
    quotaon -v "$target_mount" >/dev/null 2>&1 || true
    info "Quotas enabled."
}

set_user_quota() {
    local username=$1
    local quota_gb=${2:-0} # 0 = unlimited

    if [ "$quota_gb" -eq 0 ]; then
        info "Setting unlimited quota for $username"
        # 0 0 0 0 = unlimited
        # Need to know mount point
        local target_mount="/"
        if mountpoint -q /home; then target_mount="/home"; fi
        setquota -u "$username" 0 0 0 0 "$target_mount"
        return
    fi

    # Calculate blocks (1KB blocks)
    # 1GB = 1024 * 1024 KB = 1048576 blocks
    local blocks=$(echo "$quota_gb * 1024 * 1024" | bc)
    # Soft limit same as hard limit for now
    local soft=$blocks
    local hard=$blocks

    info "Setting quota for $username: ${quota_gb}GB ($blocks blocks)"

    local target_mount="/"
    if mountpoint -q /home; then
        target_mount="/home"
    fi

    setquota -u "$username" "$soft" "$hard" 0 0 "$target_mount"
}

# --- Docker Installation ---

install_docker() {
    info "Checking Docker installation..."
    if command -v docker &>/dev/null; then
        info "Docker is already installed."
        return
    fi

    info "Installing Docker..."
    # Use Docker convenience script for robust OS detection (Debian/Ubuntu)
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    rm get-docker.sh

    # Start and Enable Docker
    systemctl enable docker
    systemctl start docker
    info "Docker installed and started."
}

# --- Server Manager Bootstrap ---

install_server_manager() {
    info "Bootstrapping Server Manager Application..."

    # Install Rust
    if ! command -v cargo &> /dev/null; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Source env for this session
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        elif [ -f "/root/.cargo/env" ]; then
            source "/root/.cargo/env"
        fi

        # Add to path just in case
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    # Install Build Dependencies
    apt-get install -y -qq build-essential libssl-dev pkg-config

    # Clone Repository
    local src_dir="/opt/server_manager_source"
    if [ -d "$src_dir" ]; then
        info "Updating existing repository..."
        cd "$src_dir"
        git pull || info "Git pull failed, continuing with current version..."
    else
        info "Cloning repository..."
        git clone -b server-setup-script https://github.com/Cylae/server_script.git "$src_dir"
        cd "$src_dir"
    fi

    # We need to be in the subdirectory
    if [ -d "$src_dir/server_manager" ]; then
        cd "$src_dir/server_manager"
    fi

    # Build
    info "Building Server Manager (Release)..."
    cargo build --release

    # Install/Run
    info "Installation complete. Launching Server Manager..."

    # The rust app expects to be run with 'install' subcommand
    ./target/release/server_manager install
}

# Execute Setup Steps
setup_user_management
setup_security
enable_quotas
install_docker
install_server_manager

info "Server Setup Complete!"
