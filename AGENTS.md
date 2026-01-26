# AGENTS.md

## Scope
This directory contains the `server_manager` project, a Rust-based media server orchestrator.

## Principles
1.  **Rust First:** We do not use Python or Bash scripts for logic. Everything is in Rust.
2.  **Safety:** Use `Result` and `Option` extensively. Avoid `unwrap()`.
3.  **Idempotency:** All commands should be safe to run multiple times.
4.  **Hardware Awareness:** The system behaves differently based on detected hardware (Low vs High profile).

## Development
*   **Build:** `cargo build`
*   **Test:** `cargo test`
*   **Run:** `cargo run -- install`

## File Structure
*   `src/core/`: Low-level system interactions (Hardware, Docker, Firewall).
*   `src/services/`: Definitions of services (Plex, Arrs, DBs).
*   `src/main.rs`: CLI entry point and orchestration logic.

## Adding a Service
1.  Add a struct in `src/services/`.
2.  Implement the `Service` trait.
3.  Register it in `src/services/mod.rs` inside `get_all_services()`.
