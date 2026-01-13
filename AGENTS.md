# Agent Guidelines for Cylae Server Manager

## 1. Project Context
This is a modular Python-based server management framework (`Cylae/server_script`).
It is designed for Debian 11/12 and Ubuntu 20.04+ servers.
The goal is absolute stability, security, and idempotency using Docker for service deployment.

## 2. Coding Standards (Python)
* **Style:** Follow PEP 8 guidelines.
* **Type Hinting:** Use Python type hints strictly.
* **CLI:** Use `typer` for CLI commands and `rich` for output/logging.
* **Configuration:** Use `pydantic-settings` for environment configuration.
* **Root Check:** Critical functions requiring privileges should verify they are running as root (use `SystemManager.check_root()`).

## 3. Architecture
* **Core:** `src/cyl_manager/core/` contains shared logic (config, system, logging).
* **Services:** `src/cyl_manager/services/` contains service definitions inheriting from `BaseService`.
* **Registry:** Services are registered via `ServiceRegistry` in `src/cyl_manager/services/registry.py`.
* **Idempotency:** All operations must be idempotent.

## 4. Testing
* Use `pytest` for unit and integration tests located in `tests/`.
* Mock system calls (e.g., `subprocess.run`, `docker` SDK) where appropriate for unit tests.
* Run tests with `pytest`.
