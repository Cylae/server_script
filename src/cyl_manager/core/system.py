import os
import subprocess
import shutil
import platform
from pathlib import Path
from typing import Tuple, Optional, Final, Union, List
import psutil
import distro

from cyl_manager.core.logging import logger

class SystemManager:
    """
    Manager for system-level operations and checks.
    """

    _hardware_profile: HardwareProfile | None = None

    @staticmethod
    def check_root() -> None:
        """Enforces execution with elevated privileges (root)."""
        if os.geteuid() != 0:
            raise SystemRequirementError("Insufficient privileges. This operation requires root access.")

    @staticmethod
    def get_hardware_profile() -> str:
        """
        Determines the hardware profile based on system resources.

        Returns:
            str: 'LOW' if resources are constrained, else 'HIGH'.
        """
        try:
            mem = psutil.virtual_memory()
            swap = psutil.swap_memory()
            cpu_count = psutil.cpu_count() or 1

            # Criteria for LOW profile:
            # < 4GB RAM OR <= 2 Cores OR < 1GB Swap
            is_low_ram = mem.total < (4 * 1024**3)
            is_low_cpu = cpu_count <= 2
            is_low_swap = swap.total < (1 * 1024**3)

            if is_low_ram or is_low_cpu or is_low_swap:
                return SystemManager.PROFILE_LOW
            return SystemManager.PROFILE_HIGH

        except Exception as e:
            logger.warning(f"Failed to detect hardware profile, defaulting to LOW: {e}")
            return SystemManager.PROFILE_LOW

    @staticmethod
    def get_concurrency_limit() -> int:
        """
        Returns the recommended concurrency limit for operations.
        """
        return 1 if SystemManager.get_hardware_profile() == SystemManager.PROFILE_LOW else 4

    @staticmethod
    def get_uid_gid() -> Tuple[str, str]:
        """
        if SystemManager._hardware_profile is not None:
            return SystemManager._hardware_profile

        ram_gb, cpu_cores, swap_gb = SystemManager.get_system_specs()

        Returns:
            Tuple[str, str]: (UID, GID)
        """
        try:
            # If running under sudo, use the original user's ID
            sudo_uid = os.getenv("SUDO_UID")
            sudo_gid = os.getenv("SUDO_GID")

            if sudo_uid and sudo_gid:
                return sudo_uid, sudo_gid

        logger.info(f"Hardware Logic: RAM={ram_gb:.2f}GB, Cores={cpu_cores}, Swap={swap_gb:.2f}GB -> Profile={profile}")
        SystemManager._hardware_profile = profile
        return profile

    @staticmethod
    def get_timezone() -> str:
        """
        Retrieves the system timezone.

        Returns:
            str: Timezone string (e.g., 'Europe/Paris') or 'UTC' on failure.
        """
        try:
            # Check /etc/timezone first
            tz_file = Path("/etc/timezone")
            if tz_file.exists():
                return tz_file.read_text(encoding="utf-8").strip()

            # Check symlink at /etc/localtime
            localtime = Path("/etc/localtime")
            if localtime.exists() and localtime.is_symlink():
                return str(localtime.resolve()).replace("/usr/share/zoneinfo/", "")

            return "UTC"
        except Exception as e:
            logger.warning(f"Could not determine timezone: {e}. Defaulting to UTC.")
            return "UTC"

    @staticmethod
    def check_root() -> None:
        """
        Ensures the script is running with root privileges.

        Raises:
            PermissionError: If not running as root.
        """
        if os.geteuid() != 0:
            raise PermissionError("This script must be run as root!")

    @staticmethod
    def check_os() -> None:
        """
        Checks if the operating system is supported.
        Currently supports Debian/Ubuntu based systems.
        """
        try:
            os_id = distro.id()
            if os_id not in ["debian", "ubuntu", "raspbian", "linuxmint"]:
                logger.warning(f"Untested OS detected: {os_id}. Proceeding with caution.")
        except Exception:
            logger.warning("Could not detect OS distribution.")

    @staticmethod
    def check_command(command: str) -> bool:
        """
        Checks if a system command is available.

        Args:
            command: The command to check (e.g., 'docker').

        Returns:
            bool: True if available, False otherwise.
        """
        return shutil.which(command) is not None

    @staticmethod
    def run_command(command: Union[List[str], str], check: bool = True, shell: bool = False) -> subprocess.CompletedProcess:
        """
        Runs a shell command safely.

        Args:
            command: The command to execute (list of args preferred).
            check: Whether to raise an exception on non-zero exit code.
            shell: Whether to run via shell (default: False for security).

        Returns:
            subprocess.CompletedProcess: The result of the command.
        """
        logger.debug(f"Executing: {command}")
        return subprocess.run(
            command,
            shell=shell,
            check=check,
            text=True,
            capture_output=True
        )
