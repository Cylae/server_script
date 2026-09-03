use server_manager::core::journal::{
    generate_op_id, now_iso8601, CompensatoryAction, Journal, JournalEntry, StepStatus,
};
use std::collections::HashMap;
use std::fs;

#[test]
fn test_journal_creation_and_append() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_journal_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();
    let journal_path = temp_dir.join("journal.jsonl");

    let mut journal = Journal::open_or_create(&journal_path).unwrap_or_else(|e| {
        panic!("Failed to create journal: {}", e);
    });

    let op_id = generate_op_id();
    let mut params = HashMap::new();
    params.insert("target".to_string(), "nextcloud".to_string());

    let entry = JournalEntry {
        timestamp: now_iso8601(),
        op_id: op_id.clone(),
        step_index: 0,
        step_name: "deploy_service".to_string(),
        parameters: params,
        status: StepStatus::Completed,
        compensatory_action: None,
    };

    assert!(journal.append(&entry).is_ok());

    let entries = journal.read_entries().unwrap_or_default();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].op_id, op_id);
    assert_eq!(entries[0].step_name, "deploy_service");
    assert_eq!(entries[0].status, StepStatus::Completed);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_journal_compensatory_rollback_in_reverse_order() {
    let temp_dir = std::env::temp_dir().join(format!(
        "test_journal_rollback_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap_or_default();
    let journal_path = temp_dir.join("journal.jsonl");

    let mut journal = Journal::open_or_create(&journal_path).unwrap_or_else(|e| {
        panic!("Failed to create journal: {}", e);
    });

    let op_id = generate_op_id();

    // Step 0: Created file A
    let file_a = temp_dir.join("created_a.txt");
    fs::write(&file_a, "data_a").unwrap_or_default();
    assert!(file_a.exists());

    journal
        .append(&JournalEntry {
            timestamp: now_iso8601(),
            op_id: op_id.clone(),
            step_index: 0,
            step_name: "create_file_a".to_string(),
            parameters: HashMap::new(),
            status: StepStatus::Completed,
            compensatory_action: Some(CompensatoryAction::RemoveFile {
                path: file_a.clone(),
            }),
        })
        .unwrap_or_default();

    // Step 1: Created file B
    let file_b = temp_dir.join("created_b.txt");
    fs::write(&file_b, "data_b").unwrap_or_default();
    assert!(file_b.exists());

    journal
        .append(&JournalEntry {
            timestamp: now_iso8601(),
            op_id: op_id.clone(),
            step_index: 1,
            step_name: "create_file_b".to_string(),
            parameters: HashMap::new(),
            status: StepStatus::Completed,
            compensatory_action: Some(CompensatoryAction::RemoveFile {
                path: file_b.clone(),
            }),
        })
        .unwrap_or_default();

    // Step 2: Failed step
    journal
        .append(&JournalEntry {
            timestamp: now_iso8601(),
            op_id: op_id.clone(),
            step_index: 2,
            step_name: "failing_step".to_string(),
            parameters: HashMap::new(),
            status: StepStatus::Failed,
            compensatory_action: None,
        })
        .unwrap_or_default();

    // Rollback operation
    let rolled_back_count = journal.rollback_operation(&op_id).unwrap_or_default();
    assert_eq!(rolled_back_count, 2, "Both steps must be compensated");

    // Both files must have been cleaned up by compensatory actions
    assert!(!file_a.exists(), "File A must be removed by rollback");
    assert!(!file_b.exists(), "File B must be removed by rollback");

    // Check that journal now has Compensated entries
    let entries = journal.read_entries().unwrap_or_default();
    assert_eq!(entries.len(), 5); // 3 original + 2 rollback entries
    assert_eq!(entries[3].status, StepStatus::Compensated);
    assert_eq!(entries[3].step_index, 1); // step 1 compensated first
    assert_eq!(entries[4].status, StepStatus::Compensated);
    assert_eq!(entries[4].step_index, 0); // step 0 compensated second

    let _ = fs::remove_dir_all(&temp_dir);
}
