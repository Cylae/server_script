from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Type
from .system import SystemManager
from .logging import logger
from ..services.base import BaseService

class InstallationOrchestrator:
    """
    Manages the deployment of multiple services.
    Enforces concurrency limits based on system hardware profile to prevent resource exhaustion.
    """

    @staticmethod
    def install_services(services: List[BaseService]) -> None:
        """
        Installs a list of services respecting the hardware profile.

        Args:
            services: List of instantiated service objects to install.
        """
        concurrency = SystemManager.get_concurrency_limit()
        profile = SystemManager.get_hardware_profile()

        logger.info(f"Starting orchestration. Profile: {profile}. Concurrency: {concurrency} worker(s).")

        failed_services = []

        with ThreadPoolExecutor(max_workers=concurrency) as executor:
            # Map futures to service names for error reporting
            future_to_service = {
                executor.submit(service.install): service
                for service in services
                if not service.is_installed # Skip if already installed? Or maybe reinstall?
                # The prompt implies "Install... services".
                # BaseService.install() usually handles idempotency or we should check here.
                # BaseService check: "if svc.is_installed: logger.info... else: svc.install()"
                # But BaseService.install() implementation in base.py does NOT check if installed.
                # It just overwrites.
                # Let's check is_installed here to avoid unnecessary work,
                # but maybe the user wants to force update?
                # For "Clean Slate", let's assume if it's in the list, we want to ensure it's up.
                # But if it's already running, maybe we shouldn't restart it unless necessary.
                # Let's assume we proceed with install() which basically does `docker compose up -d`
                # which is idempotent if config hasn't changed.
            }

            for future in as_completed(future_to_service):
                service = future_to_service[future]
                try:
                    future.result()
                    logger.info(f"Successfully deployed: {service.pretty_name}")
                except Exception as exc:
                    logger.error(f"Failed to deploy {service.pretty_name}: {exc}")
                    failed_services.append(service.name)

        if failed_services:
            logger.warning(f"Orchestration completed with errors. Failed services: {', '.join(failed_services)}")
        else:
            logger.info("All services deployed successfully.")
