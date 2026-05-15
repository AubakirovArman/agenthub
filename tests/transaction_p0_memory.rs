use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use tempfile::TempDir;

use agenthub::agent_dir;
use agenthub::transaction::{self, TransactionStatus};

#[test]
fn no_commit_does_not_touch_main_or_promote_memory() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "noop.yaml",
        r#"
task:
  id: no_commit_preview
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'preview\n' > generated/preview.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/preview.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, true)?;

    assert!(matches!(outcome.status, TransactionStatus::Noop));
    assert!(!repo.path().join("generated/preview.txt").exists());
    assert!(outcome.report_path.exists());
    let report = fs::read_to_string(&outcome.report_path)?;
    assert!(report.contains("Status: `NOOP`"));
    let memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(!memory.contains("no_commit_preview"));
    Ok(())
}

#[test]
fn verifier_failure_rolls_back_and_does_not_promote_memory() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "verifier_failure.yaml",
        r#"
task:
  id: verifier_failure_no_promotion
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'bad\n' > generated/bad.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - sh -c 'echo verifier failed >&2; exit 1'
transaction:
  max_repair_attempts: 0
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::RolledBack));
    assert!(!repo.path().join("generated/bad.txt").exists());
    let report = fs::read_to_string(&outcome.report_path)?;
    assert!(report.contains("verifier failed"));
    let effects = fs::read_to_string(outcome.report_path.with_file_name("effects.jsonl"))?;
    assert!(effects.contains("\"status\":\"rollback_pending\""));
    assert!(effects.contains("\"status\":\"rolled_back\""));
    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(!committed_memory.contains("verifier_failure_no_promotion"));
    let failed_memory =
        fs::read_to_string(repo.path().join(".agent/memory/failed_attempts.jsonl"))?;
    assert!(failed_memory.contains("verifier_failure_no_promotion"));
    Ok(())
}

struct TestRepo {
    dir: TempDir,
    specs: TempDir,
}

impl TestRepo {
    fn new() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let specs = tempfile::tempdir()?;
        git(dir.path(), &["init"])?;
        git(dir.path(), &["config", "user.email", "test@example.com"])?;
        git(dir.path(), &["config", "user.name", "AgentHub Test"])?;
        fs::write(
            dir.path().join(".gitignore"),
            ".agent/tx/\n.agent/workspaces/\n.agent/memory/*.jsonl\n",
        )?;
        fs::write(dir.path().join("README.md"), "test project\n")?;
        Ok(Self { dir, specs })
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn commit_all(&self, message: &str) -> Result<()> {
        git(self.path(), &["add", "-A"])?;
        git(self.path(), &["commit", "-m", message])
    }

    fn write_spec(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self.specs.path().join(name);
        fs::write(&path, content.trim_start())?;
        Ok(path)
    }
}

fn git(root: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
