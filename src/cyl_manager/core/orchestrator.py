import concurrent.futures
from typing import List, Type
from ..services.base import BaseService
from ..services.registry import ServiceRegistry
from .system import SystemManager
from .logging import logger

class InstallationOrchestrator:
    def __init__(self):
        self.profile = SystemManager.get_hardware_profile()
        self.max_workers = 4 if self.profile == "HIGH" else 1

    def install_services(self, service_names: List[str]):
        """
        Installs a list of services handling dependencies and concurrency.
        """
        logger.info(f"Starting installation for {len(service_names)} services.")
        logger.info(f"Profile: {self.profile} | Concurrency: {self.max_workers} workers")

        services_to_install = []
        for name in service_names:
            cls = ServiceRegistry.get(name)
            if cls:
                services_to_install.append(cls())
            else:
                logger.warning(f"Service '{name}' not found in registry. Skipping.")

        if not services_to_install:
            logger.info("No valid services selected for installation.")
            return

        # Phase 1: Infrastructure Services (Network, DB, etc.) - ALWAYS SERIAL
        # We need to identify infrastructure. For now, we assume 'mariadb' and 'nginx-proxy' are infra.
        # But `BaseService` doesn't have a type.
        # A simple approach: Install everything serially if LOW.
        # If HIGH, we can be smarter, but let's stick to the prompt's request:
        # "Serialize installations on low-end hardware ... use parallel installation only on high-end systems."

        if self.profile == "LOW":
            self._install_serial(services_to_install)
        else:
            self._install_parallel(services_to_install)

    def _install_serial(self, services: List[BaseService]):
        for svc in services:
            try:
                if not svc.is_installed:
                    svc.install()
                    # Wait for health is handled inside install() now (next step of plan)
                else:
                    logger.info(f"{svc.pretty_name} is already installed.")
            except Exception as e:
                logger.error(f"Failed to install {svc.name}: {e}")
                # We might want to stop on error or continue?
                # "The script must pass... if any service hangs or crashes... refine"
                # For batch install, we likely want to try others unless it's critical.

    def _install_parallel(self, services: List[BaseService]):
        # We must be careful about dependencies.
        # Ideally, we define dependencies in Service class.
        # For this exercise, we assume mostly independent or Docker Compose handles `depends_on`.
        # HOWEVER, `depends_on` only works within a single compose file.
        # Here we have separate compose files.
        # So we SHOULD install Infrastructure first.

        infra_names = ["mariadb", "nginx-proxy", "mailserver"]
        infra_services = [s for s in services if s.name in infra_names]
        app_services = [s for s in services if s.name not in infra_names]

        # Install infra first (serially to be safe)
        if infra_services:
            logger.info("Installing infrastructure services first...")
            self._install_serial(infra_services)

        # Install apps in parallel
        if app_services:
            logger.info("Installing application services in parallel...")
            with concurrent.futures.ThreadPoolExecutor(max_workers=self.max_workers) as executor:
                future_to_svc = {executor.submit(self._safe_install, svc): svc for svc in app_services}
                for future in concurrent.futures.as_completed(future_to_svc):
                    svc = future_to_svc[future]
                    try:
                        future.result()
                    except Exception as e:
                        logger.error(f"Error installing {svc.name}: {e}")

    def _safe_install(self, svc: BaseService):
        """Wrapper to catch exceptions in thread."""
        if not svc.is_installed:
            svc.install()
        else:
            logger.info(f"{svc.pretty_name} is already installed.")
