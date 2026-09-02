# AGENTS.md

## Scope
`server_manager` — Rust orchestrator for a self-hosted Docker media/cloud stack.
Authoritative spec: `docs/spec/00-MISSION.md` … `05-DELIVERY.md`. This file is a summary;
on conflict, the numbered spec wins.

## Non-negotiable rules
1. Rust-first for all logic. No Bash, no Python for orchestration logic.
   (If a shell hook is ever introduced, `shellcheck -s bash` and `shfmt -i 2 -ci -bn`
   become mandatory CI gates in the same PR.)
2. No `unwrap()` / `expect()` / `panic!()` on any path reachable from user input,
   filesystem state, missing binaries, or network. Enforced by
   `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` at `deny` in non-test code.
3. Every externally-visible operation is idempotent. Running it twice changes nothing
   the second time and never errors.
4. All writes to config/secrets/compose are atomic (tmpfile on same mount → fsync →
   rename) and guarded by an advisory lock.
5. External processes are spawned with an explicit argument vector. `sh -c` with any
   interpolated value is forbidden.
6. Secrets never appear in argv, logs, error messages, HTTP responses or CI output.
7. Generated artifacts are byte-stable: generate twice, `cmp` must succeed.
8. Destructive git commands are forbidden (see 00-MISSION.md §L0.2).

## Workflow
- Build: `cargo build`
- Test: `cargo test --all-features`
- Lint: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
- Full gate: `./verify.sh` (must be a thin wrapper over the CI job — no divergence)

## Layout
- `src/core/`   : hardware, system, docker, compose, config, secrets, users, firewall
                  (+ new: atomic_io, lock, journal, sandbox)
- `src/services/`: service catalogue (infra, media, arr, download, apps)
- `src/interface/`: CLI + embedded web admin
- `docs/spec/`  : normative specification
- `docs/audit/` : evidence produced by the agent

## Adding a service
1. Struct in `src/services/`, implement the `Service` trait.
2. Register in `src/services/mod.rs::get_all_services()`.
3. Add its ports to the port matrix test; update `docs/PORT-MATRIX.md`.
4. Add a golden-file test for its compose fragment.
