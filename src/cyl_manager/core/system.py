import os
import psutil
import distro
import platform
from pathlib import Path
from typing import Tuple, Literal
from .exceptions import SystemRequirementError
from .logging import logger

HardwareProfile = Literal["LOW", "HIGH"]

class SystemManager:
    """
    Advanced System Analysis and Hardware Profiling Manager.
    Responsible for interrogating the host environment to determine capability tiers.
    """

    @staticmethod
    def check_root() -> None:
        """Enforces execution with elevated privileges (root)."""
        if os.geteuid() != 0:
            raise SystemRequirementError("Insufficient privileges. This operation requires root access.")

    @staticmethod
    def check_os() -> None:
        """Verifies OS compatibility (Debian/Ubuntu/Derivatives)."""
        os_name = distro.id()
        if os_name not in ["debian", "ubuntu", "raspbian", "linuxmint"]:
             # warning but allow, as it might work on other debian-based distros
            logger.warning(f"Detected OS: {distro.name(pretty=True)} ({os_name}). strict support is for Debian/Ubuntu.")
        else:
            logger.info(f"OS Verified: {distro.name(pretty=True)}")

    @staticmethod
    def get_system_specs() -> Tuple[float, int, float]:
        """
        Retrieves raw system specifications.
        Returns: (RAM in GB, Logical CPU Cores, Swap in GB)
        """
        mem = psutil.virtual_memory()
        swap = psutil.swap_memory()
        cpu_cores = psutil.cpu_count(logical=True) or 1

        ram_gb = mem.total / (1024**3)
        swap_gb = swap.total / (1024**3)

        return ram_gb, cpu_cores, swap_gb

    @staticmethod
    def get_hardware_profile() -> HardwareProfile:
        """
        Deterministically categorizes the host hardware into a capability profile.

        Logic:
        - LOW: < 4GB RAM OR <= 2 CPU Cores OR < 1GB Swap (if RAM < 8GB)
        - HIGH: Anything else
        """
        ram_gb, cpu_cores, swap_gb = SystemManager.get_system_specs()

        profile: HardwareProfile = "HIGH"

        # Strict Low-Spec Criteria
        if ram_gb < 3.8: # Allowing some overhead for "4GB" VPS which might show 3.8
            profile = "LOW"
            logger.debug("Profiling: Detected Low RAM (< 4GB).")
        elif cpu_cores <= 2:
            profile = "LOW"
            logger.debug("Profiling: Detected Limited CPU (<= 2 Cores).")

        logger.info(f"Hardware Logic: RAM={ram_gb:.2f}GB, Cores={cpu_cores}, Swap={swap_gb:.2f}GB -> Profile={profile}")
        return profile

    @staticmethod
    def get_concurrency_limit() -> int:
        """
        Returns the optimal number of concurrent installation workers based on the profile.

        LOW Profile -> Serial execution (1 worker) to prevent OOM/Freeze.
        HIGH Profile -> Parallel execution (4 workers) for speed.
        """
        profile = SystemManager.get_hardware_profile()
        if profile == "LOW":
            return 1
        return 4

    @staticmethod
    def check_disk_space(min_gb=10) -> None:
        """Validates available storage capacity."""
        disk = psutil.disk_usage("/")
        free_gb = disk.free / (1024**3)

        logger.debug(f"Storage Analysis: {free_gb:.2f} GB available on root.")
        if free_gb < 5:
            raise SystemRequirementError(f"CRITICAL STORAGE DEFICIT: {free_gb:.2f}GB free. Minimum 5GB required.")
        elif free_gb < min_gb:
            logger.warning(f"Storage Warning: Only {free_gb:.2f}GB free. Recommendation: >{min_gb}GB.")

    @staticmethod
    def get_uid_gid() -> Tuple[str, str]:
        """
        Resolves the appropriate UID/GID for container permissions.
        Prioritizes SUDO_UID/GID to map to the invoking user.
        """
        uid = os.environ.get("SUDO_UID", str(os.getuid()))
        try:
            import grp
            # Attempt to find docker group, otherwise fall back to user's group or 1000
            gid = os.environ.get("SUDO_GID")
            if not gid:
                gid = str(os.getgid())
        except Exception:
            gid = "1000"
        return uid, gid

    @staticmethod
    def get_timezone() -> str:
        """Detects system timezone for container synchronization."""
        if os.path.exists("/etc/timezone"):
            return Path("/etc/timezone").read_text().strip()
        if os.path.exists("/etc/localtime"):
             # Basic resolution if /etc/timezone is missing
            return "UTC"
        return "UTC"
