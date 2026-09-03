use server_manager::core::exit_codes::*;

#[test]
fn test_sysexits_constants_conform_to_standard() {
    // POSIX sysexits.h standard definitions
    assert_eq!(EX_OK, 0);
    assert_eq!(EX_USAGE, 64);
    assert_eq!(EX_DATAERR, 65);
    assert_eq!(EX_NOINPUT, 66);
    assert_eq!(EX_NOUSER, 67);
    assert_eq!(EX_SOFTWARE, 70);
    assert_eq!(EX_OSERR, 71);
    assert_eq!(EX_OSFILE, 72);
    assert_eq!(EX_CANTCREAT, 73);
    assert_eq!(EX_IOERR, 74);
    assert_eq!(EX_TEMPFAIL, 75);
    assert_eq!(EX_CONFIG, 78);
}

#[test]
fn test_map_error_to_exit_code_scenarios() {
    let err_cfg = anyhow::anyhow!("Failed to parse YAML config file");
    assert_eq!(map_error_to_exit_code(&err_cfg), EX_CONFIG);

    let err_perm = anyhow::anyhow!("Permission denied: unauthorized access to secrets");
    assert_eq!(map_error_to_exit_code(&err_perm), EX_NOINPUT);

    let err_user = anyhow::anyhow!("User 'bob' not found in user database");
    assert_eq!(map_error_to_exit_code(&err_user), EX_NOUSER);

    let err_data = anyhow::anyhow!("Invalid domain format: illegal characters");
    assert_eq!(map_error_to_exit_code(&err_data), EX_DATAERR);

    let err_io = anyhow::anyhow!("Failed to read file from disk due to IO error");
    assert_eq!(map_error_to_exit_code(&err_io), EX_IOERR);

    let err_software = anyhow::anyhow!("Internal logic error encountered");
    assert_eq!(map_error_to_exit_code(&err_software), EX_SOFTWARE);
}
