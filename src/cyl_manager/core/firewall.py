import shutil
from typing import List, Optional
from cyl_manager.core.system import SystemManager
from cyl_manager.core.logging import logger

class FirewallManager:
    """
    Manages the system firewall using ufw (Uncomplicated Firewall).
    """
    _instance: Optional['FirewallManager'] = None

    def __new__(cls) -> 'FirewallManager':
        if cls._instance is None:
            cls._instance = super(FirewallManager, cls).__new__(cls)
        return cls._instance

    @property
    def is_available(self) -> bool:
        """Checks if ufw is installed."""
        return shutil.which("ufw") is not None

    def allow_port(self, port: str, comment: str = "") -> None:
        """
        Allows a port/protocol through the firewall.
        Args:
            port: Port string (e.g., "80", "80/tcp", "5000:5010/udp").
            comment: Optional comment for the rule.
        """
        if not self.is_available:
            logger.debug("Firewall (ufw) not detected. Skipping rule.")
            return

        # Check if already allowed to prevent spamming logs/commands
        # This is a simple check, ufw handles duplicates well but being explicit is cleaner
        # However, for simplicity and robustness we just run allow.

        cmd = ["ufw", "allow", port]
        if comment:
            cmd.extend(["comment", f"Cylae: {comment}"])

        try:
            SystemManager.run_command(cmd)
            # Ensure firewall is enabled or reloaded if needed?
            # ufw rules apply immediately if active.
            logger.info(f"Firewall: Allowed port {port}")
        except Exception as e:
            logger.error(f"Failed to allow port {port}: {e}")

    def deny_port(self, port: str) -> None:
        """
        Denies/Removes a rule for a port.
        """
        if not self.is_available:
            return

        try:
            # We use 'delete allow' to reverse the allow command
            SystemManager.run_command(["ufw", "delete", "allow", port])
            logger.info(f"Firewall: Removed rule for port {port}")
        except Exception as e:
            logger.warning(f"Failed to remove firewall rule for {port}: {e}")

    def enable(self) -> None:
        """
        Enables the firewall. Warning: Ensure SSH is allowed first!
        """
        if not self.is_available:
            return

        try:
            # Check if active
            res = SystemManager.run_command(["ufw", "status"], check=False)
            if "Status: active" in res.stdout:
                return

            logger.info("Enabling Firewall (ufw)...")
            # Force 'y' to confirm prompt
            SystemManager.run_command(["ufw", "--force", "enable"])
        except Exception as e:
            logger.error(f"Failed to enable firewall: {e}")

    def ensure_basic_rules(self) -> None:
        """
        Sets up basic rules (SSH, HTTP/S) if firewall is being set up.
        """
        if not self.is_available:
            return

        logger.info("Configuring basic firewall rules...")
        self.allow_port("ssh", "SSH Access")
        self.allow_port("22", "SSH Fallback")
