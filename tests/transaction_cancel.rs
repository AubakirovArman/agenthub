use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use tempfile::TempDir;

use agenthub::agent_dir;
use agenthub::transaction::{self, TransactionStatus};

#[test]
fn cancel_request_rolls_back_and_does_not_promote_memory() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "cancel.yaml",
        r#"
task:
  id: cancel_no_promotion
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - |
      mkdir -p generated
      printf 'cancelled\n' > generated/cancelled.txt
      printf '{"requested_at":"2026-05-15T00:00:00Z","requested_by":"test","reason":"self cancel"}' > "$AGENTHUB_TX_DIR/cancel_request.json"
      sleep 5
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/cancelled.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Canceled));
    assert!(!repo.path().join("generated/cancelled.txt").exists());
    let report = fs::read_to_string(&outcome.report_path)?;
    assert!(report.contains("Status: `CANCELED`"));
    let journal = fs::read_to_string(outcome.report_path.with_file_name("journal.jsonl"))?;
    assert!(journal.contains("CANCELED"));
    let status = fs::read_to_string(outcome.report_path.with_file_name("cancel_status.json"))?;
    assert!(status.contains("self cancel"));
    let memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(!memory.contains("cancel_no_promotion"));
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
            ".agent/tx/\n.agent/workspaces/\n",
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
