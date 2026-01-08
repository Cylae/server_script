## 2024-05-22 - [Fixed Insecure MySQL Credential Handling]
**Vulnerability:** MySQL credentials and queries containing passwords were passed as command-line arguments (`--password` and `-e "SQL..."`), which are visible to all users via `ps aux`.
**Learning:** Even if a script runs as root, leaking credentials in the process list is a bad practice and violates security layers. Using command-line arguments for sensitive data is a common pitfall.
**Prevention:** Use `--defaults-extra-file` for authentication credentials and pass sensitive SQL queries via standard input (heredoc or pipe) instead of the `-e` flag.
