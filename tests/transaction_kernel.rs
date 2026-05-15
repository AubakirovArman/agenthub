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
    assert!(outcome
        .report_path
        .with_file_name("model_call_metadata.json")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("llm_gateway_summary.json")
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
    assert!(!outcome
        .report_path
        .with_file_name("raw_context_pack.json")
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
fn dry_run_cli_adapter_writes_invocation_artifacts() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "adapter_dry_run.yaml",
        r#"
task:
  id: adapter_dry_run
  type: code.command
agent:
  adapter: codex
  model: test-model
  dry_run: true
  command_template: "codex exec --prompt-file {prompt}"
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'adapter dry run\n' > generated/adapter.txt
scope:
  allow:
    - generated/**
verify:
  profile: code_build
  commands:
    - test -f generated/adapter.txt
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
    assert!(repo.path().join("generated/adapter.txt").exists());
    assert!(outcome
        .report_path
        .with_file_name("agent_prompt_executor.md")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("adapter_invocation_executor.json")
        .exists());

    let agent_trace = fs::read_to_string(outcome.report_path.with_file_name("agent_trace.json"))?;
    let transcript =
        fs::read_to_string(outcome.report_path.with_file_name("agent_transcript.jsonl"))?;
    assert!(agent_trace.contains("codex"));
    assert!(transcript.contains("\"kind\":\"adapter\""));
    assert!(transcript.contains("\"dry_run\":true"));
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

#[test]
fn repair_attempts_are_bounded_when_unresolved() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "bounded_repair.yaml",
        r#"
task:
  id: bounded_repair
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
    - test -f generated/never-created.txt
repair:
  commands:
    - printf 'attempt\n' >> generated/attempts.txt
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

    assert!(matches!(outcome.status, TransactionStatus::RolledBack));
    assert!(!repo.path().join("generated/attempts.txt").exists());
    let repair = fs::read_to_string(outcome.report_path.with_file_name("repair.json"))?;
    assert!(repair.contains("\"attempt\": 1"));
    assert!(!repair.contains("\"attempt\": 2"));
    Ok(())
}

#[test]
fn unresolved_missing_env_blocks_on_human() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "missing_env.yaml",
        r#"
task:
  id: missing_env
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'pending env\n' > generated/env.txt
scope:
  allow:
    - generated/**
verify:
  profile: code_build
  commands:
    - sh -c "echo 'missing environment variable API_KEY' >&2; exit 1"
transaction:
  max_repair_attempts: 0
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 2
    max_lines_added: 5
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::BlockedOnHuman));
    assert!(!repo.path().join("generated/env.txt").exists());
    assert!(!outcome
        .report_path
        .with_file_name("error_fingerprint.json")
        .exists());
    Ok(())
}

#[test]
fn reviewer_gate_can_repair_before_verify_and_commit() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "reviewer_repair.yaml",
        r#"
task:
  id: reviewer_repair
  type: code.command
topology:
  kind: executor_reviewer_repair
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'needs review\n' > generated/input.txt
scope:
  allow:
    - generated/**
review:
  commands:
    - test -f generated/reviewed.txt
repair:
  commands:
    - printf 'reviewed\n' > generated/reviewed.txt
verify:
  profile: code_build
  commands:
    - test -f generated/reviewed.txt
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
    assert!(repo.path().join("generated/reviewed.txt").exists());
    assert!(outcome.report_path.with_file_name("review.json").exists());
    assert!(outcome
        .report_path
        .with_file_name("review_repair.json")
        .exists());

    let agent_trace = fs::read_to_string(outcome.report_path.with_file_name("agent_trace.json"))?;
    assert!(agent_trace.contains("reviewer"));
    assert!(agent_trace.contains("repair"));
    Ok(())
}

#[test]
fn content_workspace_uses_same_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "content.yaml",
        r#"
task:
  id: draft_content_note
  type: content.command
workspace:
  type: content.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p content/notes
    - printf 'Content transaction\n' > content/notes/note.md
scope:
  allow:
    - content/**
verify:
  profile: content_quality
  commands:
    - grep -q 'Content transaction' content/notes/note.md
transaction:
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
    assert!(repo.path().join("content/notes/note.md").exists());

    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("content_change"));
    assert!(committed_memory.contains("draft_content_note"));
    Ok(())
}

#[test]
fn data_workspace_uses_same_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "data.yaml",
        r#"
task:
  id: data_quality_report
  type: data.command
workspace:
  type: data.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p data/reports
    - "printf '{\"rows\": 12, \"status\": \"ok\"}\\n' > data/reports/quality.json"
scope:
  allow:
    - data/**
verify:
  profile: data_quality
  commands:
    - test -f data/reports/quality.json
transaction:
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
    assert!(repo.path().join("data/reports/quality.json").exists());
    let verifier = fs::read_to_string(outcome.report_path.with_file_name("verifier.json"))?;
    assert!(verifier.contains("data_json_valid"));
    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("data_change"));
    Ok(())
}

#[test]
fn infra_workspace_uses_same_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "infra.yaml",
        r#"
task:
  id: infra_plan_report
  type: infra.command
workspace:
  type: infra.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p infra/plans
    - "printf 'plan: ok\\nrisk: low\\n' > infra/plans/plan.yaml"
scope:
  allow:
    - infra/**
verify:
  profile: infra_plan
  commands:
    - test -f infra/plans/plan.yaml
transaction:
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
    assert!(repo.path().join("infra/plans/plan.yaml").exists());
    let verifier = fs::read_to_string(outcome.report_path.with_file_name("verifier.json"))?;
    assert!(verifier.contains("infra_artifacts_valid"));
    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("infra_change"));
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
