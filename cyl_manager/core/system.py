import shutil
import psutil
import os
from .logger import logger

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

def get_hardware_specs():
    """Returns a dictionary of hardware specifications."""
    mem = psutil.virtual_memory()
    swap = psutil.swap_memory()
    disk = psutil.disk_usage("/")
    return {
        "cpu_cores": psutil.cpu_count(logical=True),
        "ram_gb": round(mem.total / (1024**3), 2),
        "swap_gb": round(swap.total / (1024**3), 2),
        "disk_free_gb": round(disk.free / (1024**3), 2)
    }

def check_root():
    if os.geteuid() != 0:
        logger.error("This script must be run as root.")
        # sys.exit(1) # Allow testing without root

def determine_profile():
    """Determines the hardware profile (LOW or HIGH)."""
    specs = get_hardware_specs()
    # Criteria: < 4GB RAM or <= 2 Cores => LOW
    if specs["ram_gb"] < 4 or specs["cpu_cores"] <= 2:
        return "LOW"
    return "HIGH"
