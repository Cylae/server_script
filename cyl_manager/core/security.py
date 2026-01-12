import os
import logging
from .system import run_command, install_apt_packages

logger = logging.getLogger("Security")

def install_security_tools():
    """
    Installs UFW and Fail2Ban.
    """
    install_apt_packages(['ufw', 'fail2ban'])

def configure_firewall():
    """
    Configures UFW.
    """
    logger.info("Configuring UFW...")
    try:
        run_command("ufw default deny incoming")
        run_command("ufw default allow outgoing")
        run_command("ufw allow ssh")
        run_command("ufw allow 80/tcp")
        run_command("ufw allow 443/tcp")
        # Docker handles its own iptables, but we should be careful.
        # UFW and Docker interactions can be tricky.
        # The original script just enabled UFW.
        run_command("echo 'y' | ufw enable", shell=True)
    except Exception as e:
        logger.error(f"Failed to configure firewall: {e}")

def configure_fail2ban():
    """
    Configures Fail2Ban for SSH.
    """
    logger.info("Configuring Fail2Ban...")
    # Default fail2ban config is usually okay for SSH on Debian.
    # We ensure it's running.
    try:
        run_command("systemctl enable fail2ban")
        run_command("systemctl start fail2ban")
    except Exception as e:
        logger.error(f"Failed to configure fail2ban: {e}")
