# Manifesto

**Server Manager** is built on a few core beliefs about how home server software should work.

1.  **Zero Config:** A user shouldn't need to know what `innodb_buffer_pool_size` is. The software should figure it out based on available RAM.
2.  **Process:** The `server_manager` binary analyzes the hardware profile (e.g., "Low spec machine detected, disable transcoding") and selects the optimal service configurations.
3.  **Security:** Secrets are generated, not hardcoded. Firewalls are enabled by default. Containers are isolated.
4.  **Portability:** A single static binary (`server_manager`) is easier to distribute and manage than a Python venv with fragile dependencies.
