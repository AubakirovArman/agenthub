use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;

use agenthub::{enterprise, intent, memory, spec::AgentSpec, transaction};

use super::run_summary;

pub fn handle_ask(request: &str, output: Option<&Path>, approval_required: bool) -> Result<()> {
    let preview = intent::normalize_to_spec_with_options(
        request,
        intent::IntentOptions { approval_required },
    );
    if let Some(output) = output {
        println!("{}", intent::write_preview(&preview, output)?.display());
    } else {
        print!("{}", preview.agent_spec_yaml);
    }
    print_questions(&preview);
    Ok(())
}

pub fn handle_plan(
    root: &Path,
    request: &str,
    output: Option<&Path>,
    approval_required: bool,
) -> Result<()> {
    let preview = intent::normalize_to_spec_with_options(
        request,
        intent::IntentOptions { approval_required },
    );
    let path = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| draft_path(root, "plan"));
    println!("{}", intent::write_preview(&preview, &path)?.display());
    print_questions(&preview);
    Ok(())
}

pub fn handle_run(root: &Path, target: &str, no_commit: bool) -> Result<()> {
    let spec = resolve_run_spec(root, target)?;
    print_failed_attempt_warnings(root, &spec)?;
    let actor = enterprise::authorize(root, "transaction.run")?;
    let outcome = match transaction::run(root, &spec, no_commit) {
        Ok(outcome) => outcome,
        Err(error) => {
            enterprise::record_event(
                root,
                &actor,
                "agenthub.run",
                "transaction.run",
                "error",
                Some(spec.display().to_string()),
                json!({ "error": error.to_string() }),
            )?;
            return Err(error);
        }
    };
    enterprise::record_event(
        root,
        &actor,
        "agenthub.run",
        "transaction.run",
        outcome.status.as_str(),
        Some(spec.display().to_string()),
        json!({ "tx_id": outcome.tx_id }),
    )?;
    run_summary::print(root, &spec, &outcome)
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

fn resolve_run_spec(root: &Path, target: &str) -> Result<PathBuf> {
    let path = PathBuf::from(target);
    let resolved = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    if resolved.exists() {
        return Ok(resolved);
    }
    if looks_like_path(target) {
        return Err(anyhow!("AgentSpec file does not exist: {target}"));
    }
    let preview = intent::normalize_to_spec(target);
    let output = draft_path(root, "run");
    intent::write_preview(&preview, &output)?;
    print_questions(&preview);
    Ok(output)
}

fn print_questions(preview: &intent::IntentPreview) {
    if !preview.unknowns.is_empty() {
        eprintln!("unknowns: {}", preview.unknowns.join(", "));
    }
    if !preview.questions.is_empty() {
        eprintln!("questions:");
        for question in &preview.questions {
            eprintln!("- [{}] {}", question.id, question.question);
        }
    }
}

fn draft_path(root: &Path, prefix: &str) -> PathBuf {
    root.join(".agent").join("drafts").join(format!(
        "{prefix}-{}.yaml",
        Utc::now().format("%Y%m%d%H%M%S")
    ))
}

fn looks_like_path(target: &str) -> bool {
    target.ends_with(".yaml")
        || target.ends_with(".yml")
        || target.contains('/')
        || target.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::looks_like_path;

    #[test]
    fn separates_paths_from_natural_requests() {
        assert!(looks_like_path("examples/task.yaml"));
        assert!(!looks_like_path("add a generated health file"));
    }
}
