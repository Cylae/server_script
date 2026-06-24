1. **Optimize `.clone().unwrap_or_default()` and `.clone().unwrap_or_else(...)`**
   - In `server_manager/src/services/apps.rs`, `server_manager/src/services/infra.rs`, and `server_manager/src/interface/cli.rs`, replace instances of `.clone().unwrap_or_default()` with `.as_deref().unwrap_or_default().to_string()` or `.as_deref().unwrap_or_default()` (if the destination accepts `&str`). However, since `vars.insert` expects `String`, `to_string()` will be necessary, but using `.as_deref().unwrap_or_default().to_string()` avoids full allocation of the `Option` content just to extract it or `.clone().unwrap_or_default()`. Wait, if we use `.clone().unwrap_or_default()` on an `Option<String>`, it clones the string if it exists, and creates a new empty string if it doesn't. If we use `.as_deref().unwrap_or_default().to_string()`, it just copies the string slice. Wait, the memory states: "The project standardizes on using `.as_deref().unwrap_or_default()` and `.as_deref().unwrap_or(...)` over `.clone().unwrap_or_default()` and `.clone().unwrap_or_else(...)` when handling `Option<String>` to avoid unnecessary heap allocations. Avoid appending `.to_string()` or prepending `&` when passing to functions expecting `&str` to prevent `clippy::unnecessary_to_owned` and `clippy::needless_borrow` warnings, but `.to_string()` is correct when the target requires an owned `String` (e.g., `HashMap` insertions)."
   - Replace in `apps.rs`, `infra.rs`, `cli.rs`.

2. **Update version in Cargo.toml and README.md**
   - The memory states: "The `server_manager` project is at version `1.0.9` in `Cargo.toml`." and "The `README.md` is structured with English and French sections, includes a "Testing" section, and displays badges for "Status: Tested" and "Version: 1.0.9"."
   - Update `server_manager/Cargo.toml` to version `1.0.9`.
   - Update `README.md` to version `1.0.9`.

3. **Verify the Fixes**
   - Run `cargo clippy`.
   - Run `cargo test`.
   - Run `grep` to verify no occurrences of `.clone().unwrap_or_default()`.

4. **Complete Pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit**
   - Submit the changes.
