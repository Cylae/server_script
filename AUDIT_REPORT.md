# Codebase Audit & Hardening Report

**Date:** 2026-09-13
**Auditor:** Autonomous Principal Engineer
**Status:** Audit Complete | Hardening Deployed (Verified)

## Executive Summary
A comprehensive security and robustness audit of `server_manager` was performed, focusing on concurrency primitives, process execution safety, and error handling. Several vulnerabilities and structural defects were identified and systematically remediated.

## 1. Process Execution Path Safety
- **Finding (High):** Standard system utilities (like `docker`, `ufw`, `useradd`, `userdel`, `chpasswd`, `setquota`) were invoked directly by name (e.g., `Command::new("docker")`). This relied on the environment's `$PATH` resolution, making the application vulnerable to path substitution or `$PATH` manipulation attacks.
- **Remediation:** Enforced absolute paths for all critical system utility invocations across `core/ops.rs`, `core/doctor.rs`, `core/system.rs`, `core/firewall.rs`, and `interface/cli.rs`. For example, `Command::new("docker")` was updated to `Command::new("/usr/bin/docker")`.

## 2. Uncontrolled Concurrency Errors in Web Service
- **Finding (High):** Asynchronous background tasks using `tokio::task::spawn_blocking` across the codebase (specifically in `core/config.rs` and `core/users.rs`) incorrectly resolved internal `.await` results using `unwrap()` or silently ignored thread join failures (e.g., panics inside the closure). This exposed the web service and core orchestration components to unhandled task termination.
- **Remediation:** Rearchitected `spawn_blocking` closures to safely pass thread join errors back to the caller using `.map_err()` mapped to `anyhow::anyhow!` and combined with safe `?` resolution.

## 3. Cryptographic and Filesystem State Hazards
- **Finding (Medium):** Development usage of raw `std::fs::write` directly writing sensitive state (e.g., `/root/credentials.txt`) bypassing the atomic POSIX `fsync` infrastructure introduced potential persistence hazards. Furthermore, multiple untrusted inputs lacked precise argument boundary separation.
- **Remediation:** Integrated the project's native `crate::core::atomic_io::atomic_write_str` for state persistence and enforced explicit `--` bounds separation in internal process invocations (e.g., `web.rs` daemon spawns). Eliminated direct `unwrap()` and `expect()` usage outside of explicit test modules.

## Next Steps
All deployed changes have been systematically verified using the project's native contract testing suite (`./verify.sh`), which successfully confirmed functional integrity without introducing performance degradation.

## 4. Remaining Strict Path and Argument Boundary Defenses
- **Finding (Medium):** The initial audit remediations enforcing absolute paths missed fallbacks in secondary utilities (`df`, `apt-get`, `sysctl`, `systemctl`, `journalctl`, `timedatectl`, `git`) which would still fallback to `$PATH` if the absolute path was absent, re-introducing path substitution vulnerability. Furthermore, argument bounds (`--`) were missing in `systemctl` commands.
- **Remediation:** Removed string fallbacks for all binary lookups ensuring hard failures if the binary does not exist at the trusted absolute path. Added explicit `--` bounds to `systemctl` arguments to protect against injection.

## 5. Test File Persistence Hazards
- **Finding (Low):** Raw `std::fs::write` usages remained within `config.rs` and `journal.rs` test suites.
- **Remediation:** Replaced remaining `fs::write` calls in tests with the project native `atomic_io::atomic_write_str`.
