use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::{enterprise, intent, memory, spec::AgentSpec, transaction};

pub(super) fn run_request(root: &Path, request: &str, no_commit: bool) -> Result<String> {
    let path = write_draft(root, request)?;
    run_spec(root, &path, no_commit)
}

pub(super) fn run_spec(root: &Path, spec: &Path, no_commit: bool) -> Result<String> {
    enterprise::authorize(root, "transaction.run")?;
    print_failed_attempt_warnings(root, spec)?;
    let outcome = transaction::run(root, spec, no_commit)?;
    println!(
        "{} {} ({})",
        outcome.tx_id,
        outcome.status.as_str(),
        outcome.report_path.display()
    );
    Ok(outcome.tx_id)
}

fn print_failed_attempt_warnings(root: &Path, spec_path: &Path) -> Result<()> {
    let spec = AgentSpec::load(spec_path)?;
    let query = [
        Some(spec.task.id.as_str()),
        Some(spec.task.kind.as_str()),
        spec.task.title.as_deref(),
        spec.task.target.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    for warning in memory::failed_attempt_warnings(root, &query, 3)? {
        eprintln!("warning: similar failed attempt: {}", warning.reason);
        eprintln!("mitigation: {}", warning.mitigation);
    }
    Ok(())
}

pub(super) fn write_draft(root: &Path, request: &str) -> Result<PathBuf> {
    let preview = intent::normalize_to_spec(request);
    let path = draft_path(root);
    intent::write_preview(&preview, &path)?;
    for question in preview.questions {
        eprintln!("question [{}] {}", question.id, question.question);
    }
    Ok(path)
}

pub(super) fn resolve_run_target(root: &Path, target: &str) -> Result<PathBuf> {
    let no_flag = target.replace(" --no-commit", "").trim().to_string();
    let path = PathBuf::from(&no_flag);
    let resolved = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    if resolved.exists() {
        return Ok(resolved);
    }
    write_draft(root, &no_flag).with_context(|| format!("create draft for `{no_flag}`"))
}

fn draft_path(root: &Path) -> PathBuf {
    root.join(".agent")
        .join("drafts")
        .join(format!("shell-{}.yaml", Utc::now().format("%Y%m%d%H%M%S")))
}
