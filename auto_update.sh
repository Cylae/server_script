#!/bin/bash

export DEBIAN_FRONTEND=noninteractive

# ==============================================================================
#  CYL.AE AUTO-UPDATE SCRIPT
#  Runs in background to keep system, docker, and ssl up to date.
# ==============================================================================

LOG_FILE="/var/log/server_autoupdate.log"

# Log Rotation (Keep < 1MB)
if [ -f "$LOG_FILE" ] && [ $(stat -c%s "$LOG_FILE") -gt 1048576 ]; then
    mv "$LOG_FILE" "$LOG_FILE.old"
fi

exec 1>>$LOG_FILE 2>&1

echo "----------------------------------------------------------------"
echo "STARTING UPDATE: $(date)"

# 0. Self-Update
if [ -f /etc/cyl_manager.conf ]; then
    source /etc/cyl_manager.conf
    if [ -d "$INSTALL_DIR/.git" ]; then
        echo "[SELF] Updating Server Manager script..."
        if git -C "$INSTALL_DIR" pull -q; then
             echo "[SELF] Git pull successful. Updating installed binary..."
             cp "$INSTALL_DIR/auto_update.sh" /usr/local/bin/server_autoupdate.sh
             chmod +x /usr/local/bin/server_autoupdate.sh
        else
             echo "[SELF] Git pull failed"
        fi
    fi
fi

# 1. System Updates
echo "[SYSTEM] Updating apt packages..."
apt-get update -q && apt-get upgrade -y -q

# 2. Docker Updates (via Watchtower)
echo "[DOCKER] Updating containers..."
if docker ps >/dev/null 2>&1; then
    # Run Watchtower once to update all running containers, then remove it
    docker run --rm \
        -v /var/run/docker.sock:/var/run/docker.sock \
        containrrr/watchtower \
        --run-once --cleanup --include-restarting
else
    echo "[DOCKER] Docker not running, skipping."
fi

# 3. Cleanup Docker
echo "[DOCKER] Cleaning up unused images..."
docker image prune -f

# 4. SSL Renewal
echo "[SSL] Checking certificates..."
certbot renew --quiet --post-hook "systemctl reload nginx"

echo "UPDATE COMPLETE: $(date)"
echo "----------------------------------------------------------------"
