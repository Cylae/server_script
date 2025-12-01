#!/bin/bash

# ==============================================================================
#  CYL.AE AUTO-UPDATE SCRIPT
#  Runs in background to keep system, docker, and ssl up to date.
# ==============================================================================

LOG_FILE="/var/log/server_autoupdate.log"
exec 1>>$LOG_FILE 2>&1

echo "----------------------------------------------------------------"
echo "STARTING UPDATE: $(date)"

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
