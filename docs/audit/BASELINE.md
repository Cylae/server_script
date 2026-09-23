# Baseline Report

Executed the following commands on the initial repository state:

* `cargo test` - OK
* `cargo clippy --all-targets --all-features -- -D warnings` - OK
* `cargo fmt -- --check` - OK
* `cargo deny check` - FAIL (`deny` subcommand not found, then `advisories ok, bans ok, licenses ok, sources ok` after installation)
* `cargo audit` - OK (0 vulnerabilities found)
* `./verify.sh` - OK

The baseline state is green.
