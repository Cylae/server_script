#!/bin/bash
set -euo pipefail

# ==============================================================================
# SECURITY MODULE
# Hardening, Firewall, Fail2Ban, SSH
# ==============================================================================

harden_system() {
    msg "Starting System Hardening..."

    # 1. Update/Upgrade
    msg "Updating system packages..."
    apt-get update -q && apt-get upgrade -y -q >/dev/null

    # 2. Firewall (UFW)
    configure_firewall

    # 3. Fail2Ban
    install_fail2ban

    # 4. SSH Hardening
    harden_ssh_config

    success "System Hardening Complete."
}

configure_firewall() {
    msg "Configuring Firewall (UFW)..."

    # Install UFW if not present
    if ! command -v ufw >/dev/null; then
        apt-get install -y ufw >/dev/null
    fi

    # Reset UFW to default
    ufw --force reset >/dev/null

    # Default policies: Deny incoming, Allow outgoing
    ufw default deny incoming >/dev/null
    ufw default allow outgoing >/dev/null

    # Allow SSH (Port 22 by default, will be adjusted if SSH port changes)
    # We read current SSH port
    local current_ssh_port=$(grep "^Port " /etc/ssh/sshd_config 2>/dev/null | awk '{print $2}' || true)
    current_ssh_port=${current_ssh_port:-22}

    ufw allow "$current_ssh_port/tcp" comment 'SSH' >/dev/null

    # Allow HTTP/HTTPS
    ufw allow 80/tcp comment 'HTTP' >/dev/null
    ufw allow 443/tcp comment 'HTTPS' >/dev/null

    # Enable UFW
    echo "y" | ufw enable >/dev/null
    success "Firewall configured (Block All Incoming by default, Allowed: SSH, HTTP, HTTPS)"
}

install_fail2ban() {
    msg "Installing and Configuring Fail2Ban..."
    apt-get install -y fail2ban >/dev/null

    # Create a local jail config to override defaults
    cat <<EOF > /etc/fail2ban/jail.local
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 5
destemail = root@localhost
sender = root@localhost
mta = sendmail
action = %(action_mwl)s

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
EOF

    systemctl restart fail2ban
    systemctl enable fail2ban
    success "Fail2Ban installed and configured protecting SSH."
}

harden_ssh_config() {
    msg "Hardening SSH Configuration..."

    local ssh_config="/etc/ssh/sshd_config"
    cp "$ssh_config" "${ssh_config}.bak.$(date +%F_%T)"

    # Disable Root Login
    sed -i 's/^PermitRootLogin.*/PermitRootLogin prohibit-password/' "$ssh_config"
    if ! grep -q "^PermitRootLogin" "$ssh_config"; then
        echo "PermitRootLogin prohibit-password" >> "$ssh_config"
    fi

    # Disable Password Authentication (Key-based only)
    # WARNING: Only do this if we are sure keys are set up.
    # The prompt says "enforce key-based auth if possible".
    # We should check if authorized_keys exists.

    if [ -f "/root/.ssh/authorized_keys" ] && [ -s "/root/.ssh/authorized_keys" ]; then
        sed -i 's/^PasswordAuthentication.*/PasswordAuthentication no/' "$ssh_config"
        if ! grep -q "^PasswordAuthentication" "$ssh_config"; then
            echo "PasswordAuthentication no" >> "$ssh_config"
        fi
        msg "Password Authentication disabled (Keys found)."
    else
        warn "No SSH keys found for root. Keeping PasswordAuthentication enabled for now."
        # We enforce specific robust settings though
        sed -i 's/^PermitEmptyPasswords.*/PermitEmptyPasswords no/' "$ssh_config"
    fi

    # Restart SSH
    systemctl restart sshd
}

change_ssh_port() {
    ask "Do you want to change the default SSH port (22)? [y/N]:" change_port
    if [[ "$change_port" =~ ^[Yy]$ ]]; then
        ask "Enter new SSH Port (e.g., 2222):" SSH_PORT
        # Strict numeric validation to prevent injection
        if [[ "$SSH_PORT" =~ ^[0-9]+$ ]] && [ "$SSH_PORT" -gt 0 ] && [ "$SSH_PORT" -lt 65536 ]; then
            sed -i "s/^Port .*/Port $SSH_PORT/" /etc/ssh/sshd_config
            if ! grep -q "^Port" /etc/ssh/sshd_config; then
                echo "Port $SSH_PORT" >> /etc/ssh/sshd_config
            fi

            # Update Firewall
            ufw allow "$SSH_PORT/tcp" comment 'SSH Custom' >/dev/null
            ufw delete allow 22/tcp >/dev/null 2>&1 || true

            systemctl restart sshd
            msg "SSH Port changed to $SSH_PORT."
        else
            warn "Invalid port. keeping 22."
        fi
    fi
}
