#!/bin/bash

# ==============================================================================
#  CYL.AE SERVER MANAGER - AUTO UPDATE AGENT
#  Version: 2.0 (Enhanced Reliability)
# ==============================================================================

# Strict mode
set -u

# Constants
LOG_FILE="/var/log/server_autoupdate.log"
CONFIG_FILE="/etc/cyl_manager.conf"
LOCK_FILE="/var/run/server_autoupdate.lock"

# Redirect output to log
exec 1>>"$LOG_FILE" 2>&1

# Prevent concurrent execution
if [ -f "$LOCK_FILE" ]; then
    PID=$(cat "$LOCK_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo "$(date) - Script already running with PID $PID. Exiting."
        exit 1
    fi
fi
echo $$ > "$LOCK_FILE"
trap 'rm -f "$LOCK_FILE"' EXIT

echo "================================================================"
echo "STARTING UPDATE: $(date)"

log() { echo "$(date +'%Y-%m-%d %H:%M:%S') - $1"; }

# 1. Self-Update
if [ -f "$CONFIG_FILE" ]; then
    source "$CONFIG_FILE"
    if [ -n "${INSTALL_DIR:-}" ] && [ -d "$INSTALL_DIR/.git" ]; then
        log "[SELF] Checking for script updates in $INSTALL_DIR..."

        # Fetch only
        git -C "$INSTALL_DIR" fetch origin

        LOCAL=$(git -C "$INSTALL_DIR" rev-parse HEAD)
        REMOTE=$(git -C "$INSTALL_DIR" rev-parse @{u})

        if [ "$LOCAL" != "$REMOTE" ]; then
            log "[SELF] Update found. Pulling..."
            if git -C "$INSTALL_DIR" pull -q; then
                 log "[SELF] Git pull successful."
                 # Update binaries
                 if [ -f "$INSTALL_DIR/auto_update.sh" ]; then
                    cp "$INSTALL_DIR/auto_update.sh" /usr/local/bin/server_autoupdate.sh
                    chmod +x /usr/local/bin/server_autoupdate.sh
                 fi
                 log "[SELF] Binaries updated."
            else
                 log "[SELF] ERROR: Git pull failed."
            fi
        else
            log "[SELF] No updates found."
        fi
    else
        log "[SELF] Git repository not found or INSTALL_DIR not set."
    fi
fi

# 2. System Updates
export DEBIAN_FRONTEND=noninteractive
log "[SYSTEM] Updating apt packages..."
if apt-get update -q && apt-get upgrade -y -q; then
    log "[SYSTEM] Packages updated successfully."
else
    log "[SYSTEM] ERROR: Package update failed."
fi

# 3. Docker Updates (via Watchtower)
if command -v docker >/dev/null 2>&1 && docker ps >/dev/null 2>&1; then
    log "[DOCKER] Checking for container updates..."
    # Run Watchtower once to update all running containers
    # We use containrrr/watchtower:latest-dev for latest features or stable
    if docker run --rm \
        -v /var/run/docker.sock:/var/run/docker.sock \
        containrrr/watchtower \
        --run-once --cleanup --include-restarting; then
        log "[DOCKER] Containers updated."
    else
        log "[DOCKER] Watchtower run failed."
    fi

    # Prune
    log "[DOCKER] Cleaning up unused images..."
    docker image prune -f
else
    log "[DOCKER] Docker not running or not installed."
fi

# 4. SSL Renewal
if command -v certbot >/dev/null 2>&1; then
    log "[SSL] Checking certificates..."
    certbot renew --quiet --post-hook "systemctl reload nginx"
fi

# 5. Log Rotation
if [ -f "$LOG_FILE" ] && [ $(stat -c%s "$LOG_FILE") -gt 5242880 ]; then # 5MB
    log "[LOG] Rotating log file..."
    mv "$LOG_FILE" "$LOG_FILE.old"
    # Keep only 1 old log
fi

echo "UPDATE COMPLETE: $(date)"
echo "================================================================"
