use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use agenthub::diff_guard::DiffGuardResult;
use agenthub::spec::AgentSpec;
use agenthub::transaction::{self, TransactionStatus};

#[derive(Debug, Serialize)]
struct RunSummary {
    tx_id: String,
    status: String,
    report_path: String,
    task: Option<String>,
    provider: Option<String>,
    topology: Option<String>,
    verifier: Option<String>,
    memory_promoted: Option<bool>,
    files_changed: Option<usize>,
    next_actions: Vec<String>,
}

pub fn print(
    root: &Path,
    spec_path: &Path,
    outcome: &transaction::TransactionOutcome,
) -> Result<()> {
    println!(
        "{} {} ({})",
        outcome.tx_id,
        outcome.status.as_str(),
        outcome.report_path.display()
    );
    println!();
    println!("AgentHub transaction {}", human_status(outcome.status));
    if let Ok(spec) = AgentSpec::load(spec_path) {
        println!("Task: {}", task_label(&spec));
        println!("Provider: {}", provider_label(&spec));
        println!("Topology: {}", spec.topology.kind);
        println!("Verifier: {}", verifier_label(&spec));
        println!(
            "Memory promoted: {}",
            memory_promoted(outcome.status, &spec)
        );
    }
    if let Some(files) = changed_files(&outcome.report_path)? {
        println!("Files changed: {files}");
    }
    println!("Report: {}", outcome.report_path.display());
    println!("Explain: agenthub tx explain {}", outcome.tx_id);
    println!("Watch: agenthub tx watch {}", outcome.tx_id);
    println!(
        "Dashboard: {}",
        root.join(".agent/reports/dashboard/index.html").display()
    );
    Ok(())
}

pub fn print_json(
    root: &Path,
    spec_path: &Path,
    outcome: &transaction::TransactionOutcome,
) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&summary(root, spec_path, outcome)?)?
    );
    Ok(())
}

fn summary(
    root: &Path,
    spec_path: &Path,
    outcome: &transaction::TransactionOutcome,
) -> Result<RunSummary> {
    let spec = AgentSpec::load(spec_path).ok();
    Ok(RunSummary {
        tx_id: outcome.tx_id.clone(),
        status: outcome.status.as_str().to_string(),
        report_path: outcome.report_path.display().to_string(),
        task: spec.as_ref().map(task_label),
        provider: spec.as_ref().map(provider_label),
        topology: spec.as_ref().map(|spec| spec.topology.kind.clone()),
        verifier: spec.as_ref().map(verifier_label),
        memory_promoted: spec.as_ref().map(|spec| {
            matches!(outcome.status, TransactionStatus::Committed)
                && spec.transaction.memory_promotion == "on_success"
        }),
        files_changed: changed_files(&outcome.report_path)?,
        next_actions: vec![
            format!("agenthub tx explain {}", outcome.tx_id),
            format!("agenthub tx watch {}", outcome.tx_id),
            format!(
                "agenthub dashboard --output {}",
                root.join(".agent/reports/dashboard").display()
            ),
        ],
    })
}

fn human_status(status: TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Committed => "committed",
        TransactionStatus::RolledBack => "rolled back",
        TransactionStatus::BlockedOnHuman => "blocked on human",
        TransactionStatus::Noop => "completed without commit",
        TransactionStatus::Canceled => "canceled",
    }
}

fn task_label(spec: &AgentSpec) -> String {
    spec.task
        .title
        .clone()
        .unwrap_or_else(|| spec.task.id.clone())
}

fn provider_label(spec: &AgentSpec) -> String {
    spec.agent
        .adapter
        .clone()
        .unwrap_or_else(|| "command".to_string())
}

fn verifier_label(spec: &AgentSpec) -> String {
    let profile = spec.verify.profile.as_deref().unwrap_or("default");
    if spec.verify.commands.is_empty() {
        profile.to_string()
    } else {
        format!("{} + {} command(s)", profile, spec.verify.commands.len())
    }
}

fn memory_promoted(status: TransactionStatus, spec: &AgentSpec) -> &'static str {
    if matches!(status, TransactionStatus::Committed)
        && spec.transaction.memory_promotion == "on_success"
    {
        "yes"
    } else {
        "no"
    }
}

fn changed_files(report_path: &Path) -> Result<Option<usize>> {
    let path = report_path.with_file_name("diff_guard.json");
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let diff: DiffGuardResult = serde_json::from_str(&content)?;
    Ok(Some(diff.summary.files_changed))
}

#[cfg(test)]
mod tests {
    use super::{human_status, memory_promoted};
    use agenthub::intent;
    use agenthub::spec::AgentSpec;
    use agenthub::transaction::TransactionStatus;

    #[test]
    fn summary_labels_status_and_memory() {
        let yaml = intent::normalize_to_spec("add a generated health file").agent_spec_yaml;
        let spec: AgentSpec = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(human_status(TransactionStatus::Committed), "committed");
        assert_eq!(memory_promoted(TransactionStatus::Committed, &spec), "yes");
        assert_eq!(memory_promoted(TransactionStatus::Noop, &spec), "no");
    }
}
