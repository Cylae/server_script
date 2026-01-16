from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict
from cyl_manager.core.system import SystemManager
from cyl_manager.core.logging import logger
from cyl_manager.core.docker import DockerManager

class InstallationOrchestrator:
    """
    Manages the deployment of multiple services.
    Enforces concurrency limits based on system hardware profile to prevent resource exhaustion.
    """

    @staticmethod
    def install_services(services: List['BaseService']) -> None:
        """
        Installs a list of services respecting the hardware profile.

        Logic:
        - Low Spec: Installs serially (1 by 1) to avoid OOM.
        - High Spec: Installs in parallel (4 workers).

        Args:
            services: List of instantiated service objects to install.
        """
        # Dynamic Concurrency Check
        concurrency = SystemManager.get_concurrency_limit()
        profile = SystemManager.get_hardware_profile()

        logger.info(f"Starting orchestration. Profile: {profile}. Concurrency: {concurrency} worker(s).")

        failed_services: List[str] = []

        if not services:
            logger.info("No services provided to install.")
            return

        # Pre-flight check: Ensure network exists ONCE before spawning threads
        # This prevents race conditions and redundant API calls
        try:
            DockerManager().ensure_network()
        except Exception as e:
             logger.error(f"Failed to initialize network: {e}")
             return

        with ThreadPoolExecutor(max_workers=concurrency) as executor:
            # Map futures to service instances for error reporting
            future_to_service = {
                executor.submit(service.install): service
                for service in services
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
