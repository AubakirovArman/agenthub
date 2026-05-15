mod failure;
mod files;
#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::Result;

use crate::command_policy::CommandPolicyReport;
use crate::diff_guard::DiffGuardResult;
use crate::effects::EffectRecord;
use crate::journal::JournalEvent;
use crate::smart_sync::SmartSyncDecision;
use crate::verifier::VerifierResult;

use files::{artifact, read_json, read_jsonl, status, tx_dir};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxExplanation {
    pub tx_id: String,
    pub status: String,
    pub why: Vec<String>,
    pub what: Vec<String>,
    pub next: Vec<String>,
    pub artifacts: Vec<String>,
}

impl TxExplanation {
    pub fn render_text(&self) -> String {
        let mut out = format!("Transaction {}\nStatus: {}\n", self.tx_id, self.status);
        section(&mut out, "Why", &self.why);
        section(&mut out, "What Happened", &self.what);
        section(&mut out, "Next", &self.next);
        section(&mut out, "Artifacts", &self.artifacts);
        out
    }
}

pub fn explain(root: &Path, tx_id: &str) -> Result<TxExplanation> {
    let tx_dir = tx_dir(root, tx_id)?;
    let events = read_jsonl::<JournalEvent>(&tx_dir.join("journal.jsonl"))?;
    let mut out = TxExplanation {
        tx_id: tx_id.to_string(),
        status: status(&tx_dir)?,
        why: Vec::new(),
        what: Vec::new(),
        next: Vec::new(),
        artifacts: vec![artifact(&tx_dir, "report.md")],
    };
    explain_policy(&tx_dir, &mut out)?;
    explain_diff(&tx_dir, &mut out)?;
    explain_verifier(&tx_dir, &mut out)?;
    explain_sync(&tx_dir, &mut out)?;
    explain_effects(&tx_dir, &mut out)?;
    failure::explain(&tx_dir, &events, &mut out)?;
    explain_journal(&events, &mut out);
    defaults(&mut out);
    Ok(out)
}

fn explain_policy(tx_dir: &Path, out: &mut TxExplanation) -> Result<()> {
    let Some(policy) = read_json::<CommandPolicyReport>(&tx_dir.join("command_policy.json"))?
    else {
        return Ok(());
    };
    for item in policy.violations {
        out.why.push(format!(
            "Command policy blocked {} command `{}` as `{}`.",
            item.stage, item.command, item.kind
        ));
        if item.kind == "needs_approval" {
            out.next
                .push("Record a resolution with `agenthub tx resolve <tx> --note ...`, then run `agenthub tx resume <tx>`.".to_string());
        }
    }
    Ok(())
}

fn explain_diff(tx_dir: &Path, out: &mut TxExplanation) -> Result<()> {
    let Some(diff) = read_json::<DiffGuardResult>(&tx_dir.join("diff_guard.json"))? else {
        return Ok(());
    };
    if !diff.passed {
        out.why.extend(diff.violations.iter().cloned());
        out.next
            .push("Change the task or update scope.allow/scope.deny before retrying.".to_string());
    }
    if !diff.summary.changed_files.is_empty() {
        out.what.push(format!(
            "Changed files: {}.",
            diff.summary.changed_files.join(", ")
        ));
    }
    Ok(())
}

fn explain_verifier(tx_dir: &Path, out: &mut TxExplanation) -> Result<()> {
    let Some(verifier) = read_json::<VerifierResult>(&tx_dir.join("verifier.json"))? else {
        return Ok(());
    };
    if verifier.passed {
        out.what.push("Verifier passed.".to_string());
        return Ok(());
    }
    out.why.push("Verifier failed.".to_string());
    for command in verifier.commands.iter().filter(|command| !command.success) {
        out.why
            .push(format!("Command failed: `{}`.", command.command));
    }
    out.next.push(
        "Inspect verifier.log and logs/*.stderr.log, then retry or repair the task.".to_string(),
    );
    Ok(())
}

fn explain_sync(tx_dir: &Path, out: &mut TxExplanation) -> Result<()> {
    let Some(sync) = read_json::<SmartSyncDecision>(&tx_dir.join("sync.json"))? else {
        return Ok(());
    };
    out.what.push(format!("Sync decision: {}.", sync.decision));
    if sync.decision == "blocked_overlap" {
        out.why.push(format!(
            "Smart sync found overlapping files: {}.",
            sync.overlapping_files.join(", ")
        ));
        out.next.push(
            "Resolve overlapping files manually, then record a resolution and resume.".to_string(),
        );
    }
    Ok(())
}

fn explain_effects(tx_dir: &Path, out: &mut TxExplanation) -> Result<()> {
    let effects = read_jsonl::<EffectRecord>(&tx_dir.join("effects.jsonl"))?;
    let rolled_back = effects
        .iter()
        .filter(|item| item.status == "rolled_back")
        .count();
    if rolled_back > 0 {
        out.what.push(format!(
            "Rollback recorded {rolled_back} rolled-back effect(s)."
        ));
    }
    Ok(())
}

fn explain_journal(events: &[JournalEvent], out: &mut TxExplanation) {
    if events.is_empty() {
        return;
    }
    let states = events
        .iter()
        .map(|event| event.state.as_str())
        .collect::<Vec<_>>()
        .join(" -> ");
    out.what.push(format!("Journal states: {states}."));
}

fn defaults(out: &mut TxExplanation) {
    if out.why.is_empty() {
        out.why.push(match out.status.as_str() {
            "COMMITTED" => "No failure: transaction committed.".to_string(),
            "NOOP" | "CLOSED" => {
                "No failure: transaction completed without committing.".to_string()
            }
            "CANCELED" => "Transaction was canceled by request.".to_string(),
            _ => "No specific failure artifact was found.".to_string(),
        });
    }
    if out.next.is_empty() {
        out.next.push(match out.status.as_str() {
            "COMMITTED" => "Review the report and dashboard artifacts.".to_string(),
            "CANCELED" => {
                "Run `agenthub tx retry <tx> --from EXECUTING` if the task should continue."
                    .to_string()
            }
            _ => "Open report.md for full details.".to_string(),
        });
    }
}

fn section(out: &mut String, title: &str, items: &[String]) {
    out.push('\n');
    out.push_str(title);
    out.push_str(":\n");
    for item in items {
        out.push_str("- ");
        out.push_str(item);
        out.push('\n');
    }
}
