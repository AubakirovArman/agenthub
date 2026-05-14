use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use tempfile::TempDir;

use agenthub::agent_dir;
use agenthub::transaction::{self, TransactionStatus};

#[test]
fn successful_transaction_commits_and_promotes_memory() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "success.yaml",
        r#"
task:
  id: create_generated_file
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'ok\n' > generated/result.txt
scope:
  allow:
    - generated/**
  deny:
    - prd.md
verify:
  profile: code_build
  commands:
    - test -f generated/result.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 2
    max_lines_added: 5
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    assert!(repo.path().join("generated/result.txt").exists());
    assert!(outcome.report_path.exists());
    assert!(outcome.report_path.with_file_name("dag.json").exists());
    assert!(outcome
        .report_path
        .with_file_name("agent_trace.json")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("agent_transcript.jsonl")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("context_pack_trace.json")
        .exists());
    assert!(outcome.report_path.with_file_name("cost.json").exists());
    assert!(outcome
        .report_path
        .with_file_name("skill_trace.json")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("redacted_api.jsonl")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("memory_staging.jsonl")
        .exists());
    assert!(repo
        .path()
        .join(".agent/memory/compacted/project_state.json")
        .exists());

    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("create_generated_file"));
    Ok(())
}

#[test]
fn failed_transaction_rolls_back_and_records_failed_attempt() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "failure.yaml",
        r#"
task:
  id: denied_change
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - printf 'bad\n' > forbidden.txt
scope:
  allow:
    - generated/**
  deny:
    - forbidden.txt
verify:
  profile: code_build
  commands:
    - test -f forbidden.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 5
    max_lines_added: 5
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::RolledBack));
    assert!(!repo.path().join("forbidden.txt").exists());
    assert!(outcome.report_path.exists());
    assert!(outcome
        .report_path
        .with_file_name("error_fingerprint.json")
        .exists());

    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    let failed_memory =
        fs::read_to_string(repo.path().join(".agent/memory/failed_attempts.jsonl"))?;
    assert!(!committed_memory.contains("denied_change"));
    assert!(failed_memory.contains("denied_change"));
    Ok(())
}

#[test]
fn verifier_failure_can_be_repaired_before_commit() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "repair.yaml",
        r#"
task:
  id: repair_generated_file
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'needs repair\n' > generated/input.txt
scope:
  allow:
    - generated/**
verify:
  profile: code_build
  commands:
    - test -f generated/fixed.txt
repair:
  commands:
    - printf 'fixed\n' > generated/fixed.txt
transaction:
  max_repair_attempts: 1
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 3
    max_lines_added: 10
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    assert!(repo.path().join("generated/fixed.txt").exists());
    assert!(outcome.report_path.with_file_name("repair.json").exists());
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
        run_git(dir.path(), &["init"])?;
        run_git(dir.path(), &["config", "user.email", "test@example.com"])?;
        run_git(dir.path(), &["config", "user.name", "AgentHub Test"])?;
        fs::write(
            dir.path().join(".gitignore"),
            "target/\n.agent/tx/\n.agent/workspaces/\n.agent/cache/\n.agent/memory/*.jsonl\n",
        )?;
        fs::write(dir.path().join("prd.md"), "test project\n")?;
        Ok(Self { dir, specs })
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn commit_all(&self, message: &str) -> Result<()> {
        run_git(self.path(), &["add", "-A"])?;
        run_git(self.path(), &["commit", "-m", message])?;
        Ok(())
    }

    fn write_spec(&self, name: &str, content: &str) -> Result<std::path::PathBuf> {
        let path = self.specs.path().join(name);
        fs::write(&path, content.trim_start())?;
        Ok(path)
    }
}

fn run_git(root: &Path, args: &[&str]) -> Result<()> {
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
