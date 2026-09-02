# BASELINE.md — Gate 0 Code Metrics & Verification Baseline

## Summary
This document establishes the initial code metrics, compilation state, and test execution baseline for the repository as of Gate 0 discovery.

---

## Environment & Toolchain
- **OS / Platform**: Linux x86_64
- **Rust Toolchain**: `rustc` (stable)
- **Cargo Package**: `server_manager v1.0.9` (`server_manager/Cargo.toml`)

---

## Code Base Metrics

| Metric | Value | Proof / Source |
| :--- | :--- | :--- |
| **Crate Name** | `server_manager` | `server_manager/Cargo.toml` |
| **Package Version** | `1.0.9` | `server_manager/Cargo.toml` |
| **Core Modules** | 8 (`hardware`, `system`, `docker`, `compose`, `config`, `secrets`, `users`, `firewall`) | `server_manager/src/core/` |
| **Services Modules** | 5 (`infra`, `media`, `arr`, `download`, `apps`) | `server_manager/src/services/` |
| **Service Catalogue Count** | 28 services registered | `server_manager/src/services/mod.rs` |
| **Interface Modules** | 2 (`cli`, `web`) | `server_manager/src/interface/` |

---

## Test Execution Baseline

### Command Executed
```sh
cd server_manager && cargo check --all-targets && cargo test
```

### Output Summary
- **Unit Tests (`src/lib.rs`)**: 8 passed, 0 failed, 0 ignored.
- **Integration Tests (`tests/integration_tests.rs`)**: 7 passed, 0 failed, 0 ignored.
- **Doc Tests**: 0 passed.
- **Overall Status**: `PASSED` (Exit Code 0).

---

## Lint & Formatting Baseline

### `cargo fmt --check`
- **Status**: PASSED (0 formatting errors).

### `cargo clippy --all-targets --all-features -- -D warnings`
- **Status**: PASSED (0 warnings).
