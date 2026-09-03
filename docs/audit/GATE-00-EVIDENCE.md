# GATE-00-EVIDENCE.md — Discovery, Evidence & Verification Baseline

## Toolchain provenance

The following verification toolchain binaries have been bootstrapped and verified in the environment:

```text
OK   rustc -> /home/jules/.cargo/bin/rustc (rustc 1.94.0 (4a4ef493e 2026-03-02))
OK   cargo -> /home/jules/.cargo/bin/cargo (cargo 1.94.0 (85eff7c80 2026-01-15))
OK   rustfmt -> /home/jules/.cargo/bin/rustfmt (rustfmt 1.8.0-stable (4a4ef493e3 2026-03-02))
OK   cargo-clippy -> /home/jules/.cargo/bin/cargo-clippy (clippy 0.1.94 (4a4ef493e3 2026-03-02))
OK   cargo-llvm-cov -> /home/jules/.cargo/bin/cargo-llvm-cov (cargo-llvm-cov 0.9.0)
OK   cargo-audit -> /home/jules/.cargo/bin/cargo-audit (cargo-audit 0.22.2)
OK   cargo-deny -> /home/jules/.cargo/bin/cargo-deny (cargo-deny 0.20.2)
OK   cargo-nextest -> /home/jules/.cargo/bin/cargo-nextest (cargo-nextest 0.9.143 (60fa45f63 2026-08-04))
OK   shellcheck -> /usr/bin/shellcheck (ShellCheck - shell script analysis tool)
OK   jq -> /usr/bin/jq (jq-1.7)
```

---

## Gate 0 Checklist & Requirements Mapping

| REQ ID | Validation Command | Expected Output | Acceptance Criteria | Status |
| :--- | :--- | :--- | :--- | :--- |
| **REQ-DEL-001** | `git branch -a` | Dedicated branch `gate/00-discovery-evidence` | Branch matches naming specification | REPRODUCED |
| **REQ-TST-005** | `./verify.sh` | Zero errors across fmt, clippy, test | Exits 0 | REPRODUCED |
| **REQ-SEC-011** | `grep -n 'actions/checkout' .github/workflows/*.yml` | Pinned SHA or issue noted | Unpinned mutable tag identified | REPRODUCED |

---

## Findings Evidence & Reproduction Steps

### 1. CI Workflows & Toolchain Configs (G1 Baseline)
- **Dead CI Branch Trigger**:
  - Command: `grep -n -C 3 "branches" .github/workflows/rust.yml`
  - Command Output:
    ```text
    3:on:
    4:  push:
    5:    branches: [ "server-setup-script" ]
    6:  pull_request:
    7:    branches: [ "server-setup-script" ]
    ```
  - Remote Branch Probe: `git branch -r | grep "server-setup-script"` returned empty (`BRANCH_NOT_FOUND`). Default remote branch is `main`.
- **CI / `verify.sh` Divergence**:
  - `rust.yml` runs only `cargo build` and `cargo test`.
  - `verify.sh` runs `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`.
  - Missing in CI: formatting checks, clippy static analysis, audit, deny, coverage, MSRV, dependency caching.
- **Missing Infrastructure Files**:
  - Evaluated: `rust-toolchain.toml`, `deny.toml`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/dependabot.yml`.
  - Result: All 4 files are absent from the repository tree (`No such file or directory`).

### 2. Panic & Lint Hygiene (G2 Baseline)
- **`unwrap()` / `expect()` / `panic!()` Audit**:
  - Command: `grep -rn -E "unwrap\(|expect\(|panic!\(" server_manager/src/`
  - Total Count: 16 occurrences in `server_manager/src/`.
  - Breakdown: 7 call sites in `src/interface/web.rs` (`spawn_blocking(...).expect("Blocking task should not panic")`), plus 9 occurrences in test blocks inside `users.rs` and `secrets.rs`.
  - Absence of Lint Guard: No `#[deny(clippy::unwrap_used)]` or workspace `[lints]` configuration exists in `server_manager/Cargo.toml`.
- **Criterion Benchmark vs `panic = "abort"`**:
  - Command: `cargo bench --manifest-path server_manager/Cargo.toml --no-run`
  - Result: Target built successfully (`Finished bench profile [optimized] target(s) in 2m 32s`). `panic = "abort"` in release profile does not break `cargo bench`.

### 3. Supply Chain & Injection Defense (G5 Baseline)
- **Unpinned Docker Installation Script**:
  - Source Location: `server_manager/src/core/docker.rs:18-33`
  - Evidence:
    ```rust
    Command::new("curl")
        .args(["-fsSL", "https://get.docker.com", "-o", "get-docker.sh"])
    Command::new("sh").arg("get-docker.sh")
    ```
  - Risk: Unverified, unpinned remote script execution via `sh`.
- **`rand = "0.10.0"` Resolution**:
  - Command: `cargo build --manifest-path server_manager/Cargo.toml`
  - Result: `rand v0.10.0` resolves from crates.io and compiles without errors. API call `rand::rng().fill(&mut buffer[..])` in `server_manager/src/core/secrets.rs:114` is valid for `rand v0.10.0`.
- **`Command::new` Process Invocation Audit**:
  - Total Invocations: 23 call sites across `src/core/`, `src/services/`, `src/interface/`.
  - Finding: `server_manager/src/interface/web.rs:1099` invokes `Command::new(exe).arg(action).arg(service)` where `service` is sourced from HTTP parameters without explicit allow-list validation.

---

## Baseline Verification State
Execution of `./verify.sh`:
- `cargo fmt --manifest-path server_manager/Cargo.toml -- --check`: PASSED
- `cargo clippy --manifest-path server_manager/Cargo.toml --all-targets --all-features -- -D warnings`: PASSED
- `cargo test --manifest-path server_manager/Cargo.toml`: PASSED (8 unit tests, 7 integration tests passed)
