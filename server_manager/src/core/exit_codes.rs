//! Standard sysexits.h exit codes for automation, orchestrators, and supervisor integration.
//! Authoritative reference: REQ-OPS-003.

pub const EX_OK: i32 = 0; // Successful termination
pub const EX_USAGE: i32 = 64; // Command line usage error
pub const EX_DATAERR: i32 = 65; // Data format error
pub const EX_NOINPUT: i32 = 66; // Cannot open input
pub const EX_NOUSER: i32 = 67; // Addressee unknown / user not found
pub const EX_SOFTWARE: i32 = 70; // Internal software error
pub const EX_OSERR: i32 = 71; // System error (e.g., cannot fork)
pub const EX_OSFILE: i32 = 72; // Critical OS file missing or inaccessible
pub const EX_CANTCREAT: i32 = 73; // Cannot create output file
pub const EX_IOERR: i32 = 74; // Input/output error
pub const EX_TEMPFAIL: i32 = 75; // Temporary failure; retryable
pub const EX_CONFIG: i32 = 78; // Configuration error

/// Map an anyhow error to a standard sysexits.h code.
pub fn map_error_to_exit_code(err: &anyhow::Error) -> i32 {
    let err_str = err.to_string().to_lowercase();
    if err_str.contains("config") || err_str.contains("yaml") || err_str.contains("parse") {
        EX_CONFIG
    } else if err_str.contains("permission")
        || err_str.contains("denied")
        || err_str.contains("unauthorized")
    {
        EX_NOINPUT
    } else if err_str.contains("user")
        && (err_str.contains("not found") || err_str.contains("unknown"))
    {
        EX_NOUSER
    } else if err_str.contains("invalid") || err_str.contains("format") {
        EX_DATAERR
    } else if err_str.contains("io") || err_str.contains("read") || err_str.contains("write") {
        EX_IOERR
    } else {
        EX_SOFTWARE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes_constants() {
        assert_eq!(EX_OK, 0);
        assert_eq!(EX_USAGE, 64);
        assert_eq!(EX_DATAERR, 65);
        assert_eq!(EX_NOINPUT, 66);
        assert_eq!(EX_SOFTWARE, 70);
        assert_eq!(EX_OSERR, 71);
        assert_eq!(EX_CANTCREAT, 73);
        assert_eq!(EX_TEMPFAIL, 75);
        assert_eq!(EX_CONFIG, 78);
    }

    #[test]
    fn test_map_error_to_exit_code() {
        let err_cfg = anyhow::anyhow!("Configuration file syntax error");
        assert_eq!(map_error_to_exit_code(&err_cfg), EX_CONFIG);

        let err_perm = anyhow::anyhow!("Permission denied opening secrets");
        assert_eq!(map_error_to_exit_code(&err_perm), EX_NOINPUT);

        let err_data = anyhow::anyhow!("Invalid input value for port");
        assert_eq!(map_error_to_exit_code(&err_data), EX_DATAERR);

        let err_other = anyhow::anyhow!("Unexpected internal failure");
        assert_eq!(map_error_to_exit_code(&err_other), EX_SOFTWARE);
    }
}
