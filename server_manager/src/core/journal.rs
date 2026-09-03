use anyhow::{Context, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Lifecycle status of a transaction step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Planned,
    InProgress,
    Completed,
    Failed,
    Compensated,
    CompensationFailed,
}

/// A compensatory action to roll back an executed step in reverse order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum CompensatoryAction {
    RemoveFile {
        path: PathBuf,
    },
    RestoreFile {
        path: PathBuf,
        backup_path: PathBuf,
    },
    Custom {
        name: String,
        details: HashMap<String, String>,
    },
}

/// A forward-logging journal record in JSON Lines format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: String,
    pub op_id: String,
    pub step_index: usize,
    pub step_name: String,
    pub parameters: HashMap<String, String>,
    pub status: StepStatus,
    pub compensatory_action: Option<CompensatoryAction>,
}

pub struct Journal {
    path: PathBuf,
    file: File,
}

impl Journal {
    /// Determines the default journal path.
    pub fn default_path() -> PathBuf {
        let system_dir = Path::new("/var/lib/server_manager");
        if system_dir.exists() || Path::new("/var/lib").exists() {
            system_dir.join("journal.jsonl")
        } else {
            PathBuf::from("journal.jsonl")
        }
    }

    /// Opens or creates the journal file with 0600 permissions.
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let target = path.as_ref();
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create journal dir {}", parent.display()))?;
        }

        let mut options = OpenOptions::new();
        options.create(true).read(true).append(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let file = options
            .open(target)
            .with_context(|| format!("Failed to open journal {}", target.display()))?;

        Ok(Self {
            path: target.to_path_buf(),
            file,
        })
    }

    /// Appends a new journal entry atomically with fsync.
    pub fn append(&mut self, entry: &JournalEntry) -> Result<()> {
        let mut line = serde_json::to_string(entry).context("Failed to serialize journal entry")?;
        line.push('\n');

        self.file
            .write_all(line.as_bytes())
            .with_context(|| format!("Failed to write to journal {}", self.path.display()))?;
        self.file
            .flush()
            .with_context(|| format!("Failed to flush journal {}", self.path.display()))?;
        self.file
            .sync_all()
            .with_context(|| format!("Failed to sync journal {}", self.path.display()))?;

        Ok(())
    }

    /// Reads all entries from the journal file.
    pub fn read_entries(&self) -> Result<Vec<JournalEntry>> {
        let file = File::open(&self.path)
            .with_context(|| format!("Failed to open journal {}", self.path.display()))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (i, line_res) in reader.lines().enumerate() {
            let line = line_res.with_context(|| {
                format!("Failed to read line {} of {}", i + 1, self.path.display())
            })?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: JournalEntry = serde_json::from_str(trimmed).with_context(|| {
                format!(
                    "Malformed journal line {} in {}",
                    i + 1,
                    self.path.display()
                )
            })?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Executes a single compensatory action.
    pub fn execute_compensation(action: &CompensatoryAction) -> Result<()> {
        match action {
            CompensatoryAction::RemoveFile { path } => {
                if path.exists() {
                    fs::remove_file(path).with_context(|| {
                        format!("Rollback: failed to remove file {}", path.display())
                    })?;
                    info!("Rollback: removed file {}", path.display());
                }
                Ok(())
            }
            CompensatoryAction::RestoreFile { path, backup_path } => {
                if backup_path.exists() {
                    let content = fs::read(backup_path).with_context(|| {
                        format!("Rollback: failed to read backup {}", backup_path.display())
                    })?;
                    crate::core::atomic_io::atomic_write(path, &content, 0o600)?;
                    let _ = fs::remove_file(backup_path);
                    info!(
                        "Rollback: restored {} from backup {}",
                        path.display(),
                        backup_path.display()
                    );
                }
                Ok(())
            }
            CompensatoryAction::Custom { name, details } => {
                info!(
                    "Rollback: custom compensation '{}' executed with {:?}",
                    name, details
                );
                Ok(())
            }
        }
    }

    /// Performs compensatory rollback for a given operation ID in reverse order.
    pub fn rollback_operation(&mut self, op_id: &str) -> Result<usize> {
        let entries = self.read_entries()?;
        let op_entries: Vec<&JournalEntry> = entries.iter().filter(|e| e.op_id == op_id).collect();

        let mut completed_steps: Vec<&JournalEntry> = op_entries
            .iter()
            .copied()
            .filter(|e| e.status == StepStatus::Completed && e.compensatory_action.is_some())
            .collect();

        // Sort by step_index descending to roll back in reverse order (N-1 down to 0)
        completed_steps.sort_by_key(|a| std::cmp::Reverse(a.step_index));

        let mut compensated_count = 0;
        for step in completed_steps {
            if let Some(ref action) = step.compensatory_action {
                info!(
                    "Executing rollback compensation for op={} step={}: {}",
                    op_id, step.step_index, step.step_name
                );

                let comp_res = Self::execute_compensation(action);
                let (status, err_msg) = match comp_res {
                    Ok(()) => (StepStatus::Compensated, None),
                    Err(e) => {
                        error!(
                            "Rollback failed for op={} step={}: {:#}",
                            op_id, step.step_index, e
                        );
                        (StepStatus::CompensationFailed, Some(e.to_string()))
                    }
                };

                let mut params = step.parameters.clone();
                if let Some(err) = err_msg {
                    params.insert("compensation_error".to_string(), err);
                }

                let record = JournalEntry {
                    timestamp: now_iso8601(),
                    op_id: op_id.to_string(),
                    step_index: step.step_index,
                    step_name: format!("rollback_{}", step.step_name),
                    parameters: params,
                    status,
                    compensatory_action: None,
                };

                self.append(&record)?;
                if status == StepStatus::Compensated {
                    compensated_count += 1;
                }
            }
        }

        Ok(compensated_count)
    }

    /// Scans for any unfinished operations (crashed mid-transaction) and rolls them back.
    pub fn rollback_incomplete_transactions(&mut self) -> Result<usize> {
        let entries = self.read_entries()?;
        let mut ops: HashMap<String, Vec<&JournalEntry>> = HashMap::new();
        for entry in &entries {
            ops.entry(entry.op_id.clone()).or_default().push(entry);
        }

        let mut rolled_back_ops = 0;
        for (op_id, op_entries) in ops {
            let has_failure = op_entries.iter().any(|e| e.status == StepStatus::Failed);
            let has_inprogress = op_entries
                .iter()
                .any(|e| e.status == StepStatus::InProgress);
            let is_already_compensated = op_entries
                .iter()
                .any(|e| e.status == StepStatus::Compensated);

            if (has_failure || has_inprogress) && !is_already_compensated {
                warn!(
                    "Found incomplete transaction op={}. Triggering rollback.",
                    op_id
                );
                self.rollback_operation(&op_id)?;
                rolled_back_ops += 1;
            }
        }

        Ok(rolled_back_ops)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Helper function to return current UTC timestamp in ISO 8601 format.
pub fn now_iso8601() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// Generates a 128-bit random hex transaction ID.
pub fn generate_op_id() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}
