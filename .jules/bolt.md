# Bolt's Journal

## 2024-05-22 - Parallel Docker Pulls in Bash
**Learning:** `xargs -P` is a powerful tool for parallelizing operations in Bash scripts, but it requires careful handling of output and error streams, especially in scripts with `set -e`.
**Action:** When optimizing loop-based operations like `docker pull`, check if the operations are independent and safe to run in parallel. Use `xargs -P <N> -n 1` to run them concurrently.
