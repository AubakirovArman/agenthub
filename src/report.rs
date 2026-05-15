use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::command_runner::RunnerMetadata;
use crate::diff_guard::DiffGuardResult;
use crate::observability::CostProfile;
use crate::reviewer::ReviewResult;
use crate::smart_sync::SmartSyncDecision;
use crate::verifier::VerifierResult;
use crate::workspace::WorkspaceRuntimeMetadata;

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
    pub review: Option<ReviewResult>,
    pub verifier: Option<VerifierResult>,
    pub sync: Option<SmartSyncDecision>,
    pub workspace_runtime: Option<WorkspaceRuntimeMetadata>,
    pub runner: Option<RunnerMetadata>,
    pub cost_profile: Option<CostProfile>,
    pub error_fingerprint: Option<String>,
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
            md.push_str(&format!(
                "- Lines added: `{}`\n",
                diff_guard.summary.lines_added
            ));
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

        if let Some(review) = &self.review {
            md.push_str("\n## Reviewer\n\n");
            md.push_str(&format!("- Passed: `{}`\n", review.passed));
            for command in &review.commands {
                md.push_str(&format!(
                    "- `{}` -> success `{}` exit `{:?}` timeout `{}`\n",
                    command.command, command.success, command.exit_code, command.timed_out
                ));
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
            if let Some(domain) = &verifier.domain {
                md.push_str(&format!("- Domain checks: `{}`\n", domain.passed));
                for check in &domain.checks {
                    md.push_str(&format!(
                        "- `{}` -> success `{}` detail `{}`\n",
                        check.name, check.success, check.detail
                    ));
                }
            }
            if let Some(runtime) = &verifier.runtime_smoke {
                md.push_str(&format!("- Runtime smoke: `{}`\n", runtime.passed));
                for check in &runtime.checks {
                    md.push_str(&format!(
                        "- `{}` expected `{}` actual `{:?}`\n",
                        check.path, check.expected, check.actual
                    ));
                }
            }
        }

        if let Some(sync) = &self.sync {
            md.push_str("\n## Sync\n\n");
            md.push_str(&format!("- Decision: `{}`\n", sync.decision));
            md.push_str(&format!(
                "- Verifier rerun required: `{}`\n",
                sync.verifier_rerun_required
            ));
            if !sync.overlapping_files.is_empty() {
                md.push_str("\nOverlapping files:\n\n");
                for file in &sync.overlapping_files {
                    md.push_str(&format!("- `{file}`\n"));
                }
            }
        }

        if let Some(runtime) = &self.workspace_runtime {
            md.push_str("\n## Workspace Runtime\n\n");
            md.push_str(&format!("- Runtime: `{}`\n", runtime.runtime));
            md.push_str(&format!("- Domain: `{}`\n", runtime.domain));
            md.push_str(&format!("- Isolation: `{}`\n", runtime.isolation));
            md.push_str(&format!(
                "- Capabilities: `{}`\n",
                runtime.capabilities.join(", ")
            ));
        }

        if let Some(runner) = &self.runner {
            md.push_str("\n## Runner\n\n");
            md.push_str(&format!("- Kind: `{}`\n", runner.kind));
            md.push_str(&format!("- Trust: `{}`\n", runner.trust_level));
            md.push_str(&format!(
                "- Process control: `{}`\n",
                runner.process_control
            ));
        }

        if let Some(cost) = &self.cost_profile {
            md.push_str("\n## Observability\n\n");
            md.push_str(&format!(
                "- Estimated tokens: `{}`\n",
                cost.estimated_tokens
            ));
            md.push_str(&format!("- Total cost: `${:.6}`\n", cost.total_usd));
            if !cost.breakdown.is_empty() {
                md.push_str("\nCost breakdown:\n\n");
                for item in &cost.breakdown {
                    md.push_str(&format!(
                        "- {}: `{}` tokens, `${:.6}`\n",
                        item.label, item.estimated_tokens, item.cost_usd
                    ));
                }
            }
            md.push_str("\nGateway artifacts:\n\n");
            md.push_str("- `model_call_metadata.json`\n");
            md.push_str("- `llm_gateway_summary.json`\n");
            md.push_str("- `redacted_api.jsonl`\n");
            if let Some(fingerprint) = &self.error_fingerprint {
                md.push_str(&format!("- Error fingerprint: `{fingerprint}`\n"));
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
