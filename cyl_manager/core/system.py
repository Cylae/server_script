import os
import sys
import distro
from .utils import fatal, msg

def check_root():
    """Checks if the script is running as root."""
    if os.geteuid() != 0:
        fatal("This script must be run as root.")

def check_os():
    """Checks if the OS is supported (Debian/Ubuntu)."""
    os_name = distro.id()
    if os_name not in ["debian", "ubuntu"]:
        fatal(f"Unsupported OS: {os_name}. Only Debian and Ubuntu are supported.")
    msg(f"OS Detected: {distro.name(pretty=True)}")

def get_uid_gid():
    """Returns the sudo UID and GID or defaults."""
    uid = os.environ.get("SUDO_UID", str(os.getuid()))
    try:
        import grp
        gid = os.environ.get("SUDO_GID", str(grp.getgrnam("docker").gr_gid))
    except (KeyError, ImportError):
        gid = "1000" # Fallback
    return uid, gid

def get_timezone():
    """Gets the system timezone."""
    if os.path.exists("/etc/timezone"):
        with open("/etc/timezone", "r") as f:
            return f.read().strip()
    return "UTC"
