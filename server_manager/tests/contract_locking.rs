use server_manager::core::lock::ProcessLock;
use std::fs;

#[test]
fn test_process_lock_acquisition_and_mutual_exclusion() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_lock_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();
    let lock_file = temp_dir.join("test.lock");

    // 1. Acquire first lock
    let lock1 = ProcessLock::acquire(&lock_file, true);
    assert!(lock1.is_ok(), "First lock acquisition should succeed");

    // 2. Second non-blocking acquisition on the same file should fail
    #[cfg(unix)]
    {
        let lock2 = ProcessLock::acquire(&lock_file, true);
        assert!(
            lock2.is_err(),
            "Second non-blocking lock acquisition must fail"
        );
    }

    // 3. Drop first lock
    drop(lock1);

    // 4. Now acquisition should succeed again
    let lock3 = ProcessLock::acquire(&lock_file, true);
    assert!(
        lock3.is_ok(),
        "Lock re-acquisition after drop should succeed"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
