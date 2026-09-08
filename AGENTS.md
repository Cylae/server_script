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

## Verification workflow
A task is NOT complete when the code merely compiles. Before declaring completion:

1. Inspect the repository, existing tests, CI, and the normative specs relevant to the task.
2. Make the smallest coherent implementation; do not weaken tests or quality gates to make CI green.
3. Run the complete local verification loop:
   - Build: `cargo build`
   - Test: `cargo test --all-features`
   - Lint/format: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
   - Full gate: `./verify.sh`
4. If ANY check fails, diagnose the root cause, fix it, and rerun the failed check plus the
   relevant full gate. Repeat until everything passes. Never stop merely because a first
   attempt failed.
5. Review the final diff for correctness, regressions, accidental files, debug output,
   secrets, generated noise, and spec violations.
6. If a GitHub Actions check fails after pushing, inspect the failure, fix the code/config,
   push the fix to the EXISTING PR branch, and let CI run again. Do not create a second PR
   or a replacement branch merely to repair the first one.
7. Only report success when the complete verification gate is green or when a genuinely
   external blocker prevents verification. In that case, state the exact blocker and the
   evidence; do not pretend the task is complete.

## GitHub / Jules workflow
The repository uses PRs as temporary delivery branches, but they are disposable: the goal
is for every successful Jules change to land in `main` and for its head branch to disappear.

- Target branch is `main` unless the task explicitly specifies another target.
- Keep one coherent task on one PR. Do not create multiple PRs for successive fixes to the
  same task.
- Continue fixing the SAME PR/branch when CI or review checks fail.
- Never create a new branch solely because the current Jules PR failed a test, lint check,
  build, or CI check.
- Do not merge code that has failing required checks.
- When the PR is ready and repository permissions/settings permit it, enable GitHub auto-merge
  for the PR using the repository's configured merge strategy. Prefer squash merging for
  coherent single-task changes unless the repository policy says otherwise.
- If auto-merge cannot be enabled, explain the exact permission/repository-policy blocker;
  do not silently treat manual intervention as part of normal completion.
- After merge, the source branch should be deleted automatically by GitHub. Never accumulate
  stale Jules branches.
- Never bypass branch protection, required checks, review requirements, or security controls
  merely to make automation succeed.

## Autonomous failure-recovery loop
For implementation tasks, operate as an iterative engineering loop:

`inspect → implement → verify → diagnose → fix → verify again → CI → fix CI → CI green → merge`

Do not ask the user to perform a step that can be performed with the available repository,
CI, or Jules/GitHub capabilities. Ask only when a human decision, credential, unavailable
external service, or ambiguous product requirement is genuinely required.

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
