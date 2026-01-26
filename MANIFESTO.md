# The Server Manager Manifesto

## 1. Philosophy: State-of-the-Art over Legacy
We have discarded the old Python/Bash imperative scripts in favor of a modern, declarative, and type-safe architecture. The goal is not just to "install things," but to "orchestrate a state."

## 2. The Stack: Rust
We chose **Rust** for the core orchestration engine because:
*   **Safety:** Memory safety guarantees prevent runtime crashes during critical system operations.
*   **Performance:** Zero-cost abstractions allow for complex hardware profiling (GPU detection, RAM analysis) without startup lag.
*   **Portability:** A single static binary (`server_manager`) is easier to distribute and manage than a Python venv with fragile dependencies.

## 3. Architecture: Infrastructure as Code (IaC)
Server Manager does not simply run `docker run`. It acts as a **Compiler**:
1.  **Input:** Hardware state (RAM, CPU, GPU) + User Configuration.
2.  **Process:** The `server_manager` binary analyzes the hardware profile (e.g., "Low spec machine detected, disable transcoding") and selects the optimal service configurations.
3.  **Output:** A deterministic `docker-compose.yml` file.
4.  **Execution:** Docker takes over for the actual container management, ensuring idempotency.

## 4. Hardware Optimization
We treat hardware as a first-class citizen.
*   **Plex/Jellyfin:** Transcoding capabilities (QuickSync, NVENC) are auto-detected and passed through.
*   **Arrs:** .NET garbage collection settings are tuned based on available RAM.
*   **Databases:** Buffer pools are sized dynamically.

This is the new standard for home media server orchestration.

---
*Signed, The Architecture Team (v1.0.0 Release)*
