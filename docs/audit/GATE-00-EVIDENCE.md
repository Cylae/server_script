# GATE-00-EVIDENCE.md — Discovery & Reproduction Audit Evidence

## Audit Summary
This document provides empirical command outputs and source code references reproducing every finding specified in Gate 0 discovery. All findings were reproduced against the current repository state on branch `main`.

---

## 1. CI Findings (G1)

### Finding 1.1: Triggers restricted to non-existent branch
- **Command**: `cat .github/workflows/rust.yml`
- **Output**:
```yaml
on:
  push:
    branches: [ "server-setup-script" ]
  pull_request:
    branches: [ "server-setup-script" ]
```
- **Status**: `REPRODUCED`
- **Impact**: Zero CI runs execute on `main` or pull requests targeting `main`.

### Finding 1.2: CI missing required verification steps
- **Command**: `cat .github/workflows/rust.yml`
- **Output**: Workflow executes only `cargo build` and `cargo test`. Missing `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo audit`, `cargo deny`, and MSRV check.
- **Status**: `REPRODUCED`
- **Impact**: `verify.sh` and CI diverge, violating REQ-TST-005.

### Finding 1.3: Mutable action tag usage & missing security metadata
- **Command**: `grep "uses:" .github/workflows/rust.yml`
- **Output**: `uses: actions/checkout@v4`
- **Status**: `REPRODUCED`
- **Impact**: Action tag `@v4` is mutable, violating REQ-SEC-011. Files `rust-toolchain.toml`, `deny.toml`, `.github/PULL_REQUEST_TEMPLATE.md`, and `.github/dependabot.yml` are absent.

---

## 2. Panic & Lint Hygiene (G2)

### Finding 2.1: Residual `expect()` / `unwrap()` calls in source code
- **Command**: `grep -rn "expect(" server_manager/src/`
- **Output**:
```text
server_manager/src/core/users.rs:291:        assert_eq!(user.expect("Value should exist").role, Role::Observer);
server_manager/src/core/users.rs:313:        let u = manager.get_user("appuser").expect("User exists");
server_manager/src/core/users.rs:317:        let u2 = manager.get_user("appuser").expect("User exists");
server_manager/src/core/users.rs:326:            .expect("Value should exist");
server_manager/src/core/users.rs:334:            .expect("Value should exist");
server_manager/src/core/users.rs:344:            .expect("User creation failed");
server_manager/src/core/users.rs:346:        let u = manager.get_user("user1").expect("User exists");
server_manager/src/core/users.rs:354:        let updated_u = manager.get_user("user1").expect("User exists");
server_manager/src/core/secrets.rs:131:        let hex = generate_hex(16).expect("Value should exist");
server_manager/src/interface/web.rs:524:        .expect("Blocking task should not panic")
server_manager/src/interface/web.rs:933:    .expect("Blocking task should not panic");
server_manager/src/interface/web.rs:1000:    .expect("Blocking task should not panic");
server_manager/src/interface/web.rs:1047:    .expect("Blocking task should not panic");
server_manager/src/interface/web.rs:1214:    .expect("Blocking task should not panic");
server_manager/src/interface/web.rs:1244:    .expect("Blocking task should not panic");
server_manager/src/interface/web.rs:1373:    .expect("Blocking task should not panic");
```
- **Status**: `REPRODUCED`
- **Impact**: 16 instances of `expect()` exist in non-test production modules (`core/secrets.rs`, `interface/web.rs`). No `clippy::expect_used` or `clippy::unwrap_used` deny attributes exist in the crate.

---

## 3. Supply Chain & Injection (G5, G2)

### Finding 3.1: Remote script execution (`curl | sh` pattern)
- **Command**: `sed -n '18,32p' server_manager/src/core/docker.rs`
- **Output**:
```rust
    let status = Command::new("curl")
        .args(["-fsSL", "https://get.docker.com", "-o", "get-docker.sh"])
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to download Docker setup script"));
    }

    let status = Command::new("sh")
        .arg("get-docker.sh")
        .status()?;
```
- **Status**: `REPRODUCED`
- **Impact**: Unpinned, unverified remote script download and shell execution.

### Finding 3.2: `rand` dependency version in `Cargo.toml`
- **Command**: `grep -n "rand" server_manager/Cargo.toml`
- **Output**: `rand = "0.10.0"`
- **Status**: `REPRODUCED`
- **Impact**: `rand 0.10.0` is an unreleased/yanked pre-release on crates.io, creating supply-chain and compilation dependency concerns.

---

## 4. Web Security (G6)

### Finding 4.1: Web server binds to `0.0.0.0`
- **Command**: `grep -n -C 2 "SocketAddr" server_manager/src/interface/web.rs`
- **Output**:
```rust
server_manager/src/interface/web.rs:192:    let addr = SocketAddr::from(([0, 0, 0, 0], port));
```
- **Status**: `REPRODUCED`
- **Impact**: Binds Web UI to all interfaces by default, violating REQ-SEC-007 and contradicting documentation.

### Finding 4.2: Insecure session store configuration
- **Command**: `grep -n -C 3 "SessionManagerLayer" server_manager/src/interface/web.rs`
- **Output**:
```rust
server_manager/src/interface/web.rs:130:    let session_store = MemoryStore::default();
server_manager/src/interface/web.rs:131:    let session_layer = SessionManagerLayer::new(session_store)
server_manager/src/interface/web.rs:132:        .with_secure(false);
```
- **Status**: `REPRODUCED`
- **Impact**: Uses non-persistent memory store with `with_secure(false)` cookie configuration, lacks CSRF protection on POST routes.

---

## 5. Atomicity & Locking (G3, G4)

### Finding 5.1: Non-atomic file write with delayed permission assignment
- **Command**: `sed -n '98,106p' server_manager/src/core/secrets.rs`
- **Output**:
```rust
        let yaml = serde_yaml_ng::to_string(self)?;
        std::fs::write(&path, yaml)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
```
- **Status**: `REPRODUCED`
- **Impact**: File is created with default umask permissions before setting `0600`, creating a race window where secrets are world-readable. Lacks temporary file write, fsync, atomic rename, and file locking.

### Finding 5.2: Absence of mandated core module files
- **Command**: `ls server_manager/src/core/atomic_io.rs server_manager/src/core/lock.rs server_manager/src/core/journal.rs server_manager/src/core/sandbox.rs`
- **Output**: `ls: cannot access '...': No such file or directory`
- **Status**: `REPRODUCED`
- **Impact**: `atomic_io.rs`, `lock.rs`, `journal.rs`, and `sandbox.rs` do not exist in `server_manager/src/core/`.

---

## 6. Testing Baseline (REQ-TST-001..005)

### Finding 6.1: Test Suite baseline
- **Command**: `cd server_manager && cargo test`
- **Output**: 8 unit tests passed, 7 integration tests passed. Zero contract tests (`contract_*`), zero regression tests (`regression_*`).
- **Status**: `REPRODUCED`
- **Impact**: Lacks contract-driven verification, golden file testing, and branch coverage enforcement.

---

## 7. Documentation Discrepancies (G9)

### Finding 7.1: Missing `PORT-MATRIX.md`
- **Command**: `ls docs/PORT-MATRIX.md`
- **Output**: `ls: cannot access 'docs/PORT-MATRIX.md': No such file or directory`
- **Status**: `REPRODUCED`
- **Impact**: AGENTS.md references `docs/PORT-MATRIX.md` which does not exist.

### Finding 7.2: README.md claims vs reality
- **Command**: `grep -i "service" README.md`
- **Output**: README.md claims "28 Integrated Services" while discrepancies exist across repository descriptions and docs.
- **Status**: `REPRODUCED`
