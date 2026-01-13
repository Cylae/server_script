# Agent Guidelines for Cylae Server Manager (Python Edition)

## 1. Project Context
This is a **Python-based** server management application designed for Debian-based systems (Debian 11/12, Ubuntu 20.04+).
It automates the deployment of a self-hosted media and service stack using Docker Compose V2.

## 2. Coding Standards (Python)
*   **PEP 8:** Follow standard Python style guidelines.
*   **Structure:**
    *   `cyl_manager/core/`: Shared logic (config, system, docker, security).
    *   `cyl_manager/services/`: Service implementations inheriting from `BaseService`.
    *   `cyl_manager/cli.py`: The command-line interface.
*   **Type Hinting:** Use type hints where possible to improve clarity.
*   **Imports:** Avoid circular imports. Core utilities should be imported into services, not vice-versa (except for specific type checking if needed).
*   **Logging:** Use `cyl_manager.core.utils` for logging (`msg`, `warn`, `error`, `success`) instead of `print` where appropriate.

## 3. Architecture
*   **Service Class:** Each service is a class in `cyl_manager/services/` inheriting from `BaseService`.
    *   Must implement `install()` and `remove()`.
    *   Must define `name`, `pretty_name`, and `port`.
*   **Docker:** Use `deploy_service` from `core.docker` to deploy. Generate `docker-compose.yml` content dynamically within the `install` method.
*   **Profile:** The system detects hardware (LOW vs HIGH) and services must respect `self.get_resource_limit()` to set memory limits.
*   **Idempotency:** Services should check if they are already installed or handle re-installation gracefully (e.g., updating config).

## 4. Testing
*   **Framework:** `pytest`.
*   **Location:** `tests/`.
*   **Mocking:** Use `unittest.mock` or `pytest` monkeypatching to mock system calls (`subprocess`, `os`, `distro`, `psutil`) to ensure tests can run in CI/container environments without specific hardware.
*   **Coverage:** Aim to test service logic branches (High/Low profile) and utility functions.

## 5. Deployment
*   **Docker Compose:** Always use `docker compose` (V2 syntax).
*   **Environment Variables:** Inject Python-calculated values into the Compose content string. Do not rely on shell expansion inside the Compose file (e.g., use `PUID={puid}` not `PUID=${SUDO_UID}`).
