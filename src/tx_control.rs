#[cfg(test)]
mod tests;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_dir::AgentPaths;
use crate::command_runner;
use crate::effects::EffectLedger;
use crate::journal::Journal;
use crate::spec::AgentSpec;
use crate::transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    pub ts: DateTime<Utc>,
    pub tx_id: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPlan {
    pub tx_id: String,
    pub requested_from: String,
    pub source_plan: PathBuf,
    pub retry_plan: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeReport {
    pub tx_id: String,
    pub resumed_tx_id: String,
    pub status: String,
    pub report_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReport {
    pub tx_id: String,
    pub requested_at: DateTime<Utc>,
    pub requested_by: String,
    pub reason: String,
}

pub fn cancel(root: &Path, tx_id: &str, requested_by: &str, reason: &str) -> Result<CancelReport> {
    let tx_dir = existing_tx_dir(root, tx_id)?;
    command_runner::write_cancel_request(&tx_dir, requested_by, reason)?;
    let report = CancelReport {
        tx_id: tx_id.to_string(),
        requested_at: Utc::now(),
        requested_by: requested_by.to_string(),
        reason: reason.to_string(),
    };
    fs::write(
        tx_dir.join("cancel.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    journal(tx_id, &tx_dir).append_data("CANCEL_REQUESTED", "cancel requested", json!(report))?;
    EffectLedger::for_tx_dir(&tx_dir).record_control("cancel", "requested", json!(report))?;
    Ok(report)
}

pub fn resolve(root: &Path, tx_id: &str, note: &str) -> Result<ResolutionRecord> {
    let tx_dir = existing_tx_dir(root, tx_id)?;
    let record = ResolutionRecord {
        ts: Utc::now(),
        tx_id: tx_id.to_string(),
        note: note.to_string(),
    };
    append_jsonl(&tx_dir.join("resolutions.jsonl"), &record)?;
    journal(tx_id, &tx_dir).append_data("RESOLVED", "human resolution recorded", json!(record))?;
    EffectLedger::for_tx_dir(&tx_dir).record_control("resolve", "verified", json!(record))?;
    Ok(record)
}

pub fn retry(root: &Path, tx_id: &str, from_state: &str) -> Result<RetryPlan> {
    let tx_dir = existing_tx_dir(root, tx_id)?;
    let source = tx_dir.join("plan.yaml");
    let retry_path = tx_dir.join(format!("retry-from-{}.yaml", sanitize(from_state)));
    fs::copy(&source, &retry_path).with_context(|| format!("copy {}", source.display()))?;
    let plan = RetryPlan {
        tx_id: tx_id.to_string(),
        requested_from: from_state.to_string(),
        source_plan: source,
        retry_plan: retry_path,
    };
    fs::write(
        tx_dir.join("retry_plan.json"),
        serde_json::to_string_pretty(&plan)?,
    )?;
    journal(tx_id, &tx_dir).append_data("RETRY_PLANNED", "retry plan created", json!(plan))?;
    EffectLedger::for_tx_dir(&tx_dir).record_control("retry", "planned", json!(plan))?;
    Ok(plan)
}

pub fn resume(root: &Path, tx_id: &str) -> Result<ResumeReport> {
    let tx_dir = existing_tx_dir(root, tx_id)?;
    ensure_resumable(&tx_dir)?;
    let mut spec = AgentSpec::load(&tx_dir.join("plan.yaml"))?;
    spec.transaction.approval_required = true;
    let resume_plan = tx_dir.join("resume-plan.yaml");
    fs::write(&resume_plan, serde_yaml::to_string(&spec)?)?;

    journal(tx_id, &tx_dir).append("RESUME_REQUESTED", "resume requested")?;
    EffectLedger::for_tx_dir(&tx_dir).record_control(
        "resume",
        "planned",
        json!({ "resume_plan": resume_plan.display().to_string() }),
    )?;
    let outcome = transaction::run(root, &resume_plan, false)?;
    let report = ResumeReport {
        tx_id: tx_id.to_string(),
        resumed_tx_id: outcome.tx_id,
        status: outcome.status.as_str().to_string(),
        report_path: outcome.report_path,
    };
    fs::write(
        tx_dir.join("resume.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    journal(tx_id, &tx_dir).append_data("RESUMED", "resume transaction finished", json!(report))?;
    EffectLedger::for_tx_dir(&tx_dir).record_control("resume", "verified", json!(report))?;
    Ok(report)
}

fn ensure_resumable(tx_dir: &Path) -> Result<()> {
    if !journal_contains_state(tx_dir, "BLOCKED_ON_HUMAN")? {
        return Err(anyhow!("transaction is not blocked on human"));
    }
    if fs::read_to_string(tx_dir.join("resolutions.jsonl"))
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return Err(anyhow!("transaction has no resolution note"));
    }
    Ok(())
}

fn journal_contains_state(tx_dir: &Path, state: &str) -> Result<bool> {
    let content = fs::read_to_string(tx_dir.join("journal.jsonl"))?;
    Ok(content.contains(&format!("\"state\":\"{state}\"")))
}

fn existing_tx_dir(root: &Path, tx_id: &str) -> Result<PathBuf> {
    let tx_dir = AgentPaths::new(root).tx_dir(tx_id);
    tx_dir
        .is_dir()
        .then_some(tx_dir)
        .ok_or_else(|| anyhow!("unknown transaction: {tx_id}"))
}

fn journal(tx_id: &str, tx_dir: &Path) -> Journal {
    Journal::new(tx_id, tx_dir.join("journal.jsonl"))
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}
