import os
import shutil
import logging
import re
from .shell import run_command
from .system import install_packages

logger = logging.getLogger(__name__)

def configure_firewall():
    """Configures UFW."""
    if not shutil.which("ufw"):
        install_packages(["ufw"])

    logger.info("Configuring Firewall (UFW)...")
    run_command("ufw --force reset")
    run_command("ufw default deny incoming")
    run_command("ufw default allow outgoing")

    # Detect SSH port
    ssh_port = "22"
    try:
        with open("/etc/ssh/sshd_config", "r") as f:
            for line in f:
                if line.strip().startswith("Port "):
                    ssh_port = line.strip().split()[1]
                    break
    except FileNotFoundError:
        pass

    run_command(f"ufw allow {ssh_port}/tcp comment 'SSH'")
    run_command("ufw allow 80/tcp comment 'HTTP'")
    run_command("ufw allow 443/tcp comment 'HTTPS'")

    # Enable
    run_command("ufw enable", input="y\n")
    logger.info("Firewall configured.")

def install_fail2ban():
    """Installs and configures Fail2Ban."""
    install_packages(["fail2ban"])

    jail_local = """
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
"""
    with open("/etc/fail2ban/jail.local", "w") as f:
        f.write(jail_local)

    run_command("systemctl restart fail2ban")
    run_command("systemctl enable fail2ban")
    logger.info("Fail2Ban installed.")

def harden_ssh():
    """Hardens SSH configuration."""
    logger.info("Hardening SSH...")
    config_path = "/etc/ssh/sshd_config"
    if not os.path.exists(config_path):
        return

    # Backup
    shutil.copy(config_path, f"{config_path}.bak")

    # We use regex to replace safely
    with open(config_path, 'r') as f:
        content = f.read()

    # PermitRootLogin
    if re.search(r"^PermitRootLogin", content, re.MULTILINE):
        content = re.sub(r"^PermitRootLogin.*", "PermitRootLogin prohibit-password", content, flags=re.MULTILINE)
    else:
        content += "\nPermitRootLogin prohibit-password\n"

    # PasswordAuth - check keys first
    has_keys = os.path.exists("/root/.ssh/authorized_keys") and os.path.getsize("/root/.ssh/authorized_keys") > 0

    if has_keys:
        if re.search(r"^PasswordAuthentication", content, re.MULTILINE):
             content = re.sub(r"^PasswordAuthentication.*", "PasswordAuthentication no", content, flags=re.MULTILINE)
        else:
             content += "\nPasswordAuthentication no\n"
        logger.info("Password auth disabled (keys found).")
    else:
        logger.warning("No SSH keys found for root. Keeping password auth enabled.")

    with open(config_path, 'w') as f:
        f.write(content)

    run_command("systemctl restart sshd")
