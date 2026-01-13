import os
import subprocess
from .utils import msg

def harden_system():
    """Applies security hardening (UFW, Fail2Ban)."""
    msg("Hardening System Security...")

    # 1. UFW
    subprocess.run(["ufw", "default", "deny", "incoming"], check=True)
    subprocess.run(["ufw", "default", "allow", "outgoing"], check=True)
    subprocess.run(["ufw", "allow", "ssh"], check=True)
    subprocess.run(["ufw", "allow", "http"], check=True)
    subprocess.run(["ufw", "allow", "https"], check=True)

    # Enable UFW
    # We use echo 'y' to confirm command
    subprocess.run("echo 'y' | ufw enable", shell=True, check=True)

    # 2. Fail2Ban
    if os.path.exists("/etc/fail2ban/jail.conf"):
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

        subprocess.run(["systemctl", "restart", "fail2ban"], check=True)
