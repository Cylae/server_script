# Agent Guidelines for Cylae Server Manager

## 1. Project Context
This is a production-grade Bash server management script (`Cylae/server_script`).
It is designed for Debian 11/12 and Ubuntu 20.04+ servers.
The goal is absolute stability, security, and idempotency.

## 2. Coding Standards (Bash)
* **Strict Mode:** All scripts must start with `set -euo pipefail`.
* **Variables:** Always quote variables (`"$VAR"`). Use `${VAR}` for clarity in strings.
* **Functions:** Use `snake_case` for function names.
* **Logging:** Use the `log_info`, `log_warn`, `log_error` functions defined in `src/lib/utils.sh` instead of `echo`.
* **Root Check:** Critical functions must verify they are running as root.

## 3. Architecture
* **Core:** `src/lib/` contains shared logic.
* **Modules:** `src/services/` contains install/config logic for specific services (Nginx, Docker, etc.).
* **Idempotency:** All scripts must be re-runnable without breaking the system (check if a service exists before installing).

## 4. Testing
* Use `testinfra` (Python) for integration tests located in `tests/`.
* Do not mock system commands unless strictly necessary; prefer checking exit codes.
