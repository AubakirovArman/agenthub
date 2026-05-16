use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::{enterprise, home, intent, live_run, memory, product_cli::bootstrap, spec::AgentSpec};

use super::progress;

pub(super) fn run_request(root: &Path, request: &str, no_commit: bool) -> Result<String> {
    let path = write_draft(root, request)?;
    run_spec(root, &path, no_commit)
}

pub(super) fn run_spec(root: &Path, spec: &Path, no_commit: bool) -> Result<String> {
    enterprise::authorize(root, "transaction.run")?;
    print_bootstrap(bootstrap::ensure_transaction_ready(root)?);
    print_failed_attempt_warnings(root, spec)?;
    let mut tracker = progress::default_run_tracker();
    println!("{}", tracker.render());
    tracker.complete_step(0);
    tracker.start_step(1);
    tracker.complete_step(1);
    tracker.start_step(2);
    let outcome = match live_run::run(
        root,
        spec,
        live_run::RunOptions {
            no_commit,
            watch: live_run::default_watch(),
        },
    ) {
        Ok(outcome) => {
            tracker.complete_step(2);
            tracker.start_step(3);
            tracker.complete_step(3);
            tracker.start_step(4);
            tracker.complete_step(4);
            tracker.start_step(5);
            tracker.complete_step(5);
            println!("{}", tracker.render());
            outcome
        }
        Err(error) => {
            tracker.fail_step(2);
            println!("{}", tracker.render());
            return Err(error);
        }
    };
    let elapsed = tracker.finish();
    println!(
        "{} {} ({}) in {}s",
        outcome.tx_id,
        outcome.status.as_str(),
        outcome.report_path.display(),
        elapsed.as_secs()
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
    let preview =
        intent::normalize_to_spec_for_project(root, request, intent::IntentOptions::default());
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
    let dir = if home::project_has_runtime(root) {
        root.join(".agent").join("drafts")
    } else {
        home::global_drafts_dir(root)
    };
    dir.join(format!("shell-{}.yaml", Utc::now().format("%Y%m%d%H%M%S")))
}

fn print_bootstrap(report: bootstrap::BootstrapReport) {
    if report.plan.needs_bootstrap() {
        println!("bootstrap: approved {}", report.plan.summary());
    }
    if report.git_initialized {
        println!("bootstrap: initialized git repository");
    }
    if report.agent_initialized {
        println!("bootstrap: initialized .agent project");
    }
    if report.baseline_committed {
        println!("bootstrap: committed initial AgentHub baseline");
    }
}
