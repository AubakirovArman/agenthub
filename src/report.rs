use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::diff_guard::DiffGuardResult;
use crate::verifier::VerifierResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReport {
    pub tx_id: String,
    pub task_id: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub base_head: Option<String>,
    pub committed: bool,
    pub report_path: PathBuf,
    pub diff_guard: Option<DiffGuardResult>,
    pub verifier: Option<VerifierResult>,
    pub failure_reason: Option<String>,
}

impl TransactionReport {
    pub fn write_markdown(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(path, self.to_markdown()).with_context(|| format!("write {}", path.display()))
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Transaction {}\n\n", self.tx_id));
        md.push_str(&format!("- Task: `{}`\n", self.task_id));
        md.push_str(&format!("- Status: `{}`\n", self.status));
        md.push_str(&format!("- Started: `{}`\n", self.started_at));
        md.push_str(&format!("- Finished: `{}`\n", self.finished_at));
        md.push_str(&format!(
            "- Base HEAD: `{}`\n",
            self.base_head.as_deref().unwrap_or("<none>")
        ));
        md.push_str(&format!("- Committed: `{}`\n", self.committed));

        if let Some(diff_guard) = &self.diff_guard {
            md.push_str("\n## Diff Guard\n\n");
            md.push_str(&format!("- Passed: `{}`\n", diff_guard.passed));
            md.push_str(&format!(
                "- Files changed: `{}`\n",
                diff_guard.summary.files_changed
            ));
            md.push_str(&format!("- Lines added: `{}`\n", diff_guard.summary.lines_added));
            md.push_str(&format!(
                "- Lines deleted: `{}`\n",
                diff_guard.summary.lines_deleted
            ));
            if !diff_guard.summary.changed_files.is_empty() {
                md.push_str("\nChanged files:\n\n");
                for file in &diff_guard.summary.changed_files {
                    md.push_str(&format!("- `{file}`\n"));
                }
            }
            if !diff_guard.violations.is_empty() {
                md.push_str("\nViolations:\n\n");
                for violation in &diff_guard.violations {
                    md.push_str(&format!("- {violation}\n"));
                }
            }
        }

        if let Some(verifier) = &self.verifier {
            md.push_str("\n## Verifier\n\n");
            md.push_str(&format!("- Passed: `{}`\n", verifier.passed));
            md.push_str(&format!(
                "- Profile: `{}`\n",
                verifier.profile.as_deref().unwrap_or("<none>")
            ));
            for command in &verifier.commands {
                md.push_str(&format!(
                    "- `{}` -> success `{}` exit `{:?}` timeout `{}`\n",
                    command.command, command.success, command.exit_code, command.timed_out
                ));
            }
        }

        if let Some(reason) = &self.failure_reason {
            md.push_str("\n## Failure\n\n");
            md.push_str(reason);
            md.push('\n');
        }

        md
    }
}

