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
                # We do NOT skip if installed, because "install" ensures the configuration is up-to-date.
                # docker compose up -d is idempotent.
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
