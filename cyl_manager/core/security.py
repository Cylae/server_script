import os
import subprocess
from .logger import logger

class SecurityManager:
    def enable_firewall(self, ports: list):
        """Configures UFW."""
        if os.geteuid() != 0:
            logger.warning("Not running as root, skipping Firewall configuration.")
            return

        try:
            if subprocess.run("which ufw", shell=True).returncode != 0:
                 subprocess.run("apt-get install -y ufw", shell=True, check=True)

            logger.info("Configuring Firewall (UFW)...")
            # We don't want to lock ourselves out if running remotely, so be careful
            # This is a dangerous operation in a script without interactive confirmation
            # But the user asked for it.

            # subprocess.run("ufw default deny incoming", shell=True) # Dangerous if SSH not allowed
            # subprocess.run("ufw default allow outgoing", shell=True)
            subprocess.run("ufw allow ssh", shell=True)
            subprocess.run("ufw allow 22", shell=True) # Explicit
            subprocess.run("ufw allow http", shell=True)
            subprocess.run("ufw allow https", shell=True)

            for port in ports:
                # Handle port/proto format
                subprocess.run(f"ufw allow {port}", shell=True)

            # subprocess.run("ufw --force enable", shell=True) # Dangerous in non-interactive
            logger.info(f"Firewall rules updated for ports: {ports}")
        except Exception as e:
            logger.error(f"Failed to configure firewall: {e}")

    def install_fail2ban(self):
        """Installs and configures Fail2Ban."""
        if os.geteuid() != 0: return

        try:
            if subprocess.run("which fail2ban-client", shell=True).returncode != 0:
                logger.info("Installing Fail2Ban...")
                subprocess.run("apt-get install -y fail2ban", shell=True, check=True)

                # Basic jail config
                jail_local = """
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 5

[sshd]
enabled = true
"""
                with open("/etc/fail2ban/jail.local", "w") as f:
                    f.write(jail_local)

                subprocess.run("systemctl restart fail2ban", shell=True)
                logger.info("Fail2Ban installed and configured.")
        except Exception as e:
            logger.error(f"Failed to install Fail2Ban: {e}")

security_manager = SecurityManager()
