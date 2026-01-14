# Bolt's Journal

## 2024-05-22 - Parallel Docker Pulls in Bash
**Learning:** `xargs -P` is a powerful tool for parallelizing operations in Bash scripts, but it requires careful handling of output and error streams, especially in scripts with `set -e`.
**Action:** When optimizing loop-based operations like `docker pull`, check if the operations are independent and safe to run in parallel. Use `xargs -P <N> -n 1` to run them concurrently.

## 2024-05-23 - Docker Client Instantiation Overhead
**Learning:** `docker.from_env()` performs expensive environment parsing and socket connection checks every time it is called. Using it inside a loop (indirectly via object instantiation) causes N+1 performance issues.
**Action:** Use a Singleton pattern or pass the client instance explicitly when multiple objects need access to the Docker daemon.
