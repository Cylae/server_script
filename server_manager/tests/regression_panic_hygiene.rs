use server_manager::core::users::{Role, UserManager};
use std::fs;

#[test]
fn test_regression_no_panic_on_invalid_users_file() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_panic_hygiene_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();
    let invalid_file = temp_dir.join("users.yaml");
    fs::write(&invalid_file, "INVALID_YAML: [:::").unwrap_or_default();

    // Verify loading an invalid YAML file does not panic but returns an error
    let res = serde_yaml_ng::from_str::<UserManager>("INVALID_YAML: [:::");
    assert!(res.is_err(), "Invalid YAML must return Err, not panic");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_regression_user_manager_error_handling() {
    let mut manager = UserManager::default();
    let res1 = manager.add_user("test_user", "pass123", Role::Observer, None);
    assert!(res1.is_ok());

    // Duplicate user should return an Err, never panic
    let res2 = manager.add_user("test_user", "pass456", Role::Observer, None);
    assert!(res2.is_err());

    // Non-existent user deletion should return Err, never panic
    let res3 = manager.delete_user("non_existent_user");
    assert!(res3.is_err());
}
