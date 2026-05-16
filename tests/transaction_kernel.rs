use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use tempfile::TempDir;

use agenthub::agent_dir;
use agenthub::transaction::{self, TransactionStatus};
use agenthub::tx_control;

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
    let runner = fs::read_to_string(outcome.report_path.with_file_name("runner.json"))?;
    assert!(runner.contains("\"trust_level\""));
    assert!(runner.contains("process_control"));
    assert!(outcome
        .report_path
        .with_file_name("cancel_status.json")
        .exists());
    let runtime = fs::read_to_string(outcome.report_path.with_file_name("workspace_runtime.json"))?;
    assert!(runtime.contains("CodeGitWorkspace"));
    assert!(runtime.contains("commit"));
    let report = fs::read_to_string(&outcome.report_path)?;
    assert!(report.contains("## Workspace Runtime"));
    assert!(report.contains("## Domain Runtime"));
    assert!(report.contains("## Runner"));
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
        .with_file_name("redaction_report.json")
        .exists());
    assert!(!outcome
        .report_path
        .with_file_name("raw_context_pack.json")
        .exists());
    assert!(outcome
        .report_path
        .with_file_name("memory_staging.jsonl")
        .exists());
    let effects = fs::read_to_string(outcome.report_path.with_file_name("effects.jsonl"))?;
    assert!(effects.contains("\"status\":\"verified\""));
    assert!(effects.contains("\"path\":\"generated/result.txt\""));
    assert!(outcome.report_path.with_file_name("wal.jsonl").exists());
    let wal_replay = fs::read_to_string(outcome.report_path.with_file_name("wal_replay.json"))?;
    assert!(wal_replay.contains("\"latest_state\": \"CLOSED\""));
    assert!(repo
        .path()
        .join(".agent/memory/compacted/project_state.json")
        .exists());
    assert!(repo
        .path()
        .join(".agent/memory/views/project_state.json")
        .exists());
    assert!(repo.path().join(".agent/memory/audit.json").exists());
    let domain_runtime =
        fs::read_to_string(outcome.report_path.with_file_name("domain_runtime.json"))?;
    assert!(domain_runtime.contains("code.rust"));
    assert!(repo
        .path()
        .join(".agent/metrics/analytics_history.jsonl")
        .exists());
    assert!(repo
        .path()
        .join(".agent/metrics/analytics_history.csv")
        .exists());
    let analytics = fs::read_to_string(repo.path().join(".agent/metrics/analytics_summary.json"))?;
    assert!(analytics.contains("\"success_rate\": 1.0"));
    assert!(analytics.contains("code.command"));

    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("create_generated_file"));
    assert!(committed_memory.contains("\"schema\":\"code.memory.v1\""));
    Ok(())
}

#[test]
fn api_adapter_dry_run_writes_prompt_and_invocation_without_provider_call() -> Result<()> {
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
  adapter: deepseek
  model: test-model
  dry_run: true
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
    assert!(agent_trace.contains("deepseek"));
    assert!(agent_trace.contains("\"selected_adapter\": \"deepseek\""));
    let provider_plan =
        fs::read_to_string(outcome.report_path.with_file_name("llm_provider_plan.json"))?;
    assert!(provider_plan.contains("deepseek"));
    assert!(provider_plan.contains("api_provider"));
    Ok(())
}

#[test]
fn api_adapter_executes_provider_generated_commands_inside_transaction() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;
    let response = r#"{"summary":"create generated file","commands":["mkdir -p generated","printf 'api native\n' > generated/api.txt"]}"#;
    let stub = OpenAiStub::start(response)?;

    let spec = repo.write_spec(
        "api_native_executor.yaml",
        r#"
task:
  id: api_native_executor
  type: code.command
agent:
  adapter: deepseek
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands: []
scope:
  allow:
    - generated/**
verify:
  profile: code_build
  commands:
    - test -f generated/api.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 2
    max_lines_added: 5
    max_lines_deleted: 0
"#,
    )?;

    with_deepseek_stub_env(&stub.endpoint, || {
        let outcome = transaction::run(repo.path(), &spec, false)?;

        assert!(matches!(outcome.status, TransactionStatus::Committed));
        assert_eq!(
            normalize_newlines(&fs::read_to_string(repo.path().join("generated/api.txt"))?),
            "api native\n"
        );
        assert!(outcome
            .report_path
            .with_file_name("api_execution_executor.json")
            .exists());
        let adapter = fs::read_to_string(
            outcome
                .report_path
                .with_file_name("adapter_invocation_executor.json"),
        )?;
        assert!(adapter.contains("api://deepseek"));
        let request = stub.received_request()?;
        assert!(request.contains("POST /v1/chat/completions"));
        Ok(())
    })
}

#[test]
fn api_adapter_reinjects_builtin_tool_results_before_command_plan() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    fs::write(
        repo.path().join("instructions.txt"),
        "create api tool output\n",
    )?;
    repo.commit_all("agenthub baseline")?;
    let tool_response = r#"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"call-read","type":"function","function":{"name":"read_file","arguments":"{\"path\":\"instructions.txt\"}"}}]}}],"usage":{"completion_tokens":7}}"#;
    let plan_response = OpenAiStub::content_body(
        r#"{"summary":"create generated file after reading","commands":["mkdir -p generated","printf 'tool reinjected\n' > generated/tool-loop.txt"]}"#,
    );
    let stub = OpenAiStub::start_bodies(vec![tool_response.to_string(), plan_response])?;

    let spec = repo.write_spec(
        "api_tool_reinject.yaml",
        r#"
task:
  id: api_tool_reinject
  type: code.command
agent:
  adapter: deepseek
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands: []
scope:
  allow:
    - generated/**
verify:
  profile: code_build
  commands:
    - test -f generated/tool-loop.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 2
    max_lines_added: 5
    max_lines_deleted: 0
"#,
    )?;

    with_deepseek_stub_env(&stub.endpoint, || {
        let outcome = transaction::run(repo.path(), &spec, false)?;

        assert!(matches!(outcome.status, TransactionStatus::Committed));
        assert_eq!(
            normalize_newlines(&fs::read_to_string(
                repo.path().join("generated/tool-loop.txt")
            )?),
            "tool reinjected\n"
        );
        let receipt = fs::read_to_string(
            outcome
                .report_path
                .with_file_name("tool_results_executor.json"),
        )?;
        assert!(receipt.contains("call-read"));
        assert!(receipt.contains("create api tool output"));
        let first_request = stub.received_request()?;
        let second_request = stub.received_request()?;
        assert!(first_request.contains("read_file"), "{first_request}");
        assert!(
            second_request.contains("AgentHub builtin tool results"),
            "{second_request}"
        );
        Ok(())
    })
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
    let effects = fs::read_to_string(outcome.report_path.with_file_name("effects.jsonl"))?;
    assert!(effects.contains("\"status\":\"rolled_back\""));
    assert!(effects.contains("\"path\":\"forbidden.txt\""));
    let rollback = fs::read_to_string(outcome.report_path.with_file_name("rollback.json"))?;
    assert!(rollback.contains("\"handler\": \"file_restore\""));
    let runtime = fs::read_to_string(outcome.report_path.with_file_name("workspace_runtime.json"))?;
    assert!(runtime.contains("CodeGitWorkspace"));
    assert!(runtime.contains("rollback"));

    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    let failed_memory =
        fs::read_to_string(repo.path().join(".agent/memory/failed_attempts.jsonl"))?;
    assert!(!committed_memory.contains("denied_change"));
    assert!(failed_memory.contains("denied_change"));
    Ok(())
}

#[test]
fn command_policy_blocks_needs_approval_without_flag() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "needs_approval.yaml",
        r#"
task:
  id: needs_approval_command
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - npm install left-pad
scope:
  allow:
    - package.json
transaction:
  commit_on_success: true
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::BlockedOnHuman));
    let policy = fs::read_to_string(outcome.report_path.with_file_name("command_policy.json"))?;
    assert!(policy.contains("needs_approval"));
    Ok(())
}

#[test]
fn resolve_and_resume_blocked_transaction() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    fs::write(
        repo.path().join(".agent/policies/core.yaml"),
        "commands:\n  needs_approval:\n    - printf\n",
    )?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "resume_blocked.yaml",
        r#"
task:
  id: resume_blocked
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'approved\n' > generated/resumed.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/resumed.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
"#,
    )?;

    let blocked = transaction::run(repo.path(), &spec, false)?;
    assert!(matches!(blocked.status, TransactionStatus::BlockedOnHuman));
    tx_control::resolve(repo.path(), &blocked.tx_id, "approved printf")?;
    let resumed = tx_control::resume(repo.path(), &blocked.tx_id)?;

    assert_eq!(resumed.status, "COMMITTED");
    assert!(repo.path().join("generated/resumed.txt").exists());
    assert!(blocked.report_path.with_file_name("resume.json").exists());
    let effects = fs::read_to_string(blocked.report_path.with_file_name("effects.jsonl"))?;
    assert!(effects.contains("control:resolve"));
    assert!(effects.contains("control:resume"));
    Ok(())
}

#[test]
fn smart_sync_rebases_independent_main_changes() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    fs::create_dir_all(repo.path().join("generated"))?;
    fs::write(repo.path().join("generated/existing.txt"), "baseline\n")?;
    repo.commit_all("agenthub baseline")?;
    let root = shell_path(repo.path());
    let spec = repo.write_spec(
        "smart_sync_rebase.yaml",
        &format!(
            r#"
task:
  id: smart_sync_rebase
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p {root}/notes
    - printf 'main\n' > {root}/notes/main.txt
    - git -C {root} add notes/main.txt
    - git -C {root} commit -m concurrent-main
    - mkdir -p generated
    - printf 'tx\n' > generated/tx.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/tx.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
"#
        ),
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    assert!(repo.path().join("notes/main.txt").exists());
    assert!(repo.path().join("generated/tx.txt").exists());
    let sync = fs::read_to_string(outcome.report_path.with_file_name("sync.json"))?;
    assert!(sync.contains("rebase_required"));
    let baseline = fs::read_to_string(outcome.report_path.with_file_name("baseline.json"))?;
    assert!(baseline.contains("\"scoped_files\""));
    assert!(baseline.contains("generated/existing.txt"));
    let report = fs::read_to_string(outcome.report_path)?;
    assert!(report.contains("## Sync"));
    Ok(())
}

#[test]
fn smart_sync_blocks_overlapping_main_changes() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;
    let root = shell_path(repo.path());
    let spec = repo.write_spec(
        "smart_sync_overlap.yaml",
        &format!(
            r#"
task:
  id: smart_sync_overlap
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p {root}/generated
    - printf 'main\n' > {root}/generated/conflict.txt
    - git -C {root} add generated/conflict.txt
    - git -C {root} commit -m concurrent-overlap
    - mkdir -p generated
    - printf 'tx\n' > generated/conflict.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/conflict.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
"#
        ),
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::BlockedOnHuman));
    assert_eq!(
        fs::read_to_string(repo.path().join("generated/conflict.txt"))?,
        "main\n"
    );
    let sync = fs::read_to_string(outcome.report_path.with_file_name("sync.json"))?;
    assert!(sync.contains("blocked_overlap"));
    assert!(sync.contains("generated/conflict.txt"));
    Ok(())
}

#[test]
fn command_policy_rejects_restricted_command() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "restricted.yaml",
        r#"
task:
  id: restricted_command
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - rm -rf generated
scope:
  allow:
    - generated/**
transaction:
  commit_on_success: true
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::RolledBack));
    let policy = fs::read_to_string(outcome.report_path.with_file_name("command_policy.json"))?;
    assert!(policy.contains("restricted"));
    let failed_memory =
        fs::read_to_string(repo.path().join(".agent/memory/failed_attempts.jsonl"))?;
    assert!(failed_memory.contains("restricted_command"));
    Ok(())
}

#[test]
fn sandbox_level_one_runs_with_sandbox_metadata() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "sandbox_level_1.yaml",
        r#"
task:
  id: sandbox_level_one
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  sandbox:
    level: 1
  commands:
    - test "$AGENTHUB_SANDBOX_LEVEL" = 1
scope:
  allow:
    - generated/**
transaction:
  commit_on_success: false
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Noop));
    let sandbox = fs::read_to_string(outcome.report_path.with_file_name("sandbox.json"))?;
    assert!(sandbox.contains("local_sandbox"));
    Ok(())
}

#[test]
fn resource_policy_timeout_is_enforced_for_execution_commands() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    fs::write(
        repo.path().join(".agent/policies/resources.yaml"),
        "resources:\n  timeout_secs: 1\n  network: inherit\n  filesystem: workspace\n",
    )?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "resource_timeout.yaml",
        r#"
task:
  id: resource_timeout
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - sleep 5
scope:
  allow:
    - generated/**
transaction:
  commit_on_success: true
  memory_promotion: on_success
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::RolledBack));
    let runner = fs::read_to_string(outcome.report_path.with_file_name("runner.json"))?;
    assert!(runner.contains("\"timeout_secs\": 1"));
    let execution = fs::read_to_string(outcome.report_path.with_file_name("execution.json"))?;
    assert!(execution.contains("\"timed_out\": true"));
    Ok(())
}

#[test]
fn sandbox_level_two_blocks_for_external_runner() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "sandbox_level_2.yaml",
        r#"
task:
  id: sandbox_level_two
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  sandbox:
    level: 2
  commands:
    - true
scope:
  allow:
    - generated/**
transaction:
  commit_on_success: true
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::BlockedOnHuman));
    let sandbox = fs::read_to_string(outcome.report_path.with_file_name("sandbox.json"))?;
    assert!(sandbox.contains("remote_runner_required"));
    Ok(())
}

#[test]
fn remote_runner_dispatch_collects_results() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    fs::write(
        repo.path().join(".agent/enterprise/policy.yaml"),
        "enterprise:\n  runners:\n    default: local\n    remote:\n      - id: local-remote\n        endpoint: local://runner\n        labels:\n          - strong-isolation\n",
    )?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "remote_runner.yaml",
        r#"
task:
  id: remote_runner_demo
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  sandbox:
    level: 2
  commands:
    - mkdir -p generated
    - printf "$AGENTHUB_REMOTE_RUNNER\n" > generated/remote.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/remote.txt
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
    assert_eq!(
        normalize_newlines(&fs::read_to_string(
            repo.path().join("generated/remote.txt")
        )?),
        "local-remote\n"
    );
    let execution = fs::read_to_string(outcome.report_path.with_file_name("execution.json"))?;
    assert!(execution.contains("\"remote\": true"));
    assert!(execution.contains("local-remote"));
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
    let structured = fs::read_to_string(
        outcome
            .report_path
            .with_file_name("verifier_integration.json"),
    )?;
    assert!(structured.contains("\"fingerprints\""));
    assert!(structured.contains("\"check_id\": \"command-0\""));
    let report = fs::read_to_string(&outcome.report_path)?;
    assert!(report.contains("Structured checks"));
    assert!(report.contains("Verifier fingerprints"));
    let failed_memory =
        fs::read_to_string(repo.path().join(".agent/memory/failed_attempts.jsonl"))?;
    assert!(failed_memory.contains("fingerprint"));
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

#[test]
fn media_workspace_uses_same_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "media.yaml",
        r#"
task:
  id: media_render_plan
  type: media.command
workspace:
  type: media.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p media/renders
    - "printf '{\"scene\":\"intro\",\"format\":\"mp4\"}\\n' > media/manifest.json"
    - printf 'video-bytes\n' > media/renders/intro.mp4
scope:
  allow:
    - media/**
verify:
  profile: media_render
  commands:
    - test -f media/manifest.json
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 4
    max_lines_added: 10
    max_lines_deleted: 0
"#,
    )?;

    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    assert!(repo.path().join("media/manifest.json").exists());
    let verifier = fs::read_to_string(outcome.report_path.with_file_name("verifier.json"))?;
    assert!(verifier.contains("media_manifests_valid"));
    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("media_change"));
    Ok(())
}

#[test]
fn research_workspace_uses_same_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec("research.yaml", research_spec())?;
    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    assert!(repo.path().join("research/report.md").exists());
    let verifier = fs::read_to_string(outcome.report_path.with_file_name("verifier.json"))?;
    assert!(verifier.contains("research_claims_cited"));
    let committed_memory = fs::read_to_string(repo.path().join(".agent/memory/committed.jsonl"))?;
    assert!(committed_memory.contains("research_change"));
    Ok(())
}

#[test]
fn manager_worker_topology_writes_agent_trace() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "manager_worker.yaml",
        r#"
task:
  id: manager_worker_demo
  type: code.command
topology:
  kind: manager_worker
  swarm_size: 2
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'managed\n' > generated/managed.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/managed.txt
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
    let trace = fs::read_to_string(outcome.report_path.with_file_name("agent_trace.json"))?;
    assert!(trace.contains("manager"));
    assert!(trace.contains("worker_2"));
    Ok(())
}

#[test]
fn adaptive_orchestration_records_decision_report_and_scoreboard() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "adaptive.yaml",
        r#"
task:
  id: adaptive_feature_demo
  type: code.command
  title: Add adaptive feature output
topology:
  kind: single_executor
  routing:
    adaptive: true
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'adaptive\n' > generated/adaptive.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/adaptive.txt
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
    let adaptive = fs::read_to_string(outcome.report_path.with_file_name("adaptive.json"))?;
    assert!(adaptive.contains("\"enabled\": true"));
    assert!(adaptive.contains("\"selected_topology\": \"manager_worker\""));
    assert!(adaptive.contains("\"inputs\""));
    let report = fs::read_to_string(&outcome.report_path)?;
    assert!(report.contains("## Adaptive Orchestration"));
    assert!(report.contains("manager_worker"));
    let scoreboard = fs::read_to_string(
        repo.path()
            .join(".agent/metrics/orchestration_scoreboard.json"),
    )?;
    assert!(scoreboard.contains("\"topology\": \"manager_worker\""));
    assert!(scoreboard.contains("\"success\": 1"));
    Ok(())
}

#[test]
fn tournament_topology_writes_agent_trace() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "tournament.yaml",
        r#"
task:
  id: tournament_demo
  type: code.command
topology:
  kind: tournament
  swarm_size: 3
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'winner\n' > generated/winner.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/winner.txt
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
    let trace = fs::read_to_string(outcome.report_path.with_file_name("agent_trace.json"))?;
    let dag = fs::read_to_string(outcome.report_path.with_file_name("dag.json"))?;
    assert!(trace.contains("contestant_3"));
    assert!(trace.contains("judge"));
    assert!(dag.contains("contestant_1"));
    assert!(dag.contains("judge"));
    Ok(())
}

#[test]
fn backend_tdd_verifier_profile_uses_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "backend_tdd.yaml",
        r#"
task:
  id: backend_tdd_demo
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p backend/tests/unit backend/tests/integration
    - printf 'unit ok\n' > backend/tests/unit/health.test.ts
    - printf 'integration ok\n' > backend/tests/integration/health.test.ts
    - "printf '{\"unit_tests\":[\"backend/tests/unit/health.test.ts\"],\"integration_tests\":[\"backend/tests/integration/health.test.ts\"],\"api_responses\":[{\"method\":\"GET\",\"path\":\"/health\",\"status\":200,\"body\":{\"ok\":true}}]}\n' > backend/tdd.json"
scope:
  allow:
    - backend/**
verify:
  profile: backend_tdd
  commands:
    - test -f backend/tests/unit/health.test.ts
    - test -f backend/tests/integration/health.test.ts
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 4
    max_lines_added: 10
    max_lines_deleted: 0
"#,
    )?;
    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    let verifier = fs::read_to_string(outcome.report_path.with_file_name("verifier.json"))?;
    assert!(verifier.contains("backend_tdd"));
    assert!(verifier.contains("backend_api_responses_valid"));
    Ok(())
}

#[test]
fn db_migration_verifier_profile_uses_transaction_kernel() -> Result<()> {
    let repo = TestRepo::new()?;
    agent_dir::init_project(repo.path(), false)?;
    repo.commit_all("agenthub baseline")?;

    let spec = repo.write_spec(
        "db_migration.yaml",
        r#"
task:
  id: db_migration_demo
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p db/migrations db/seeds
    - printf 'create table users;\n' > db/migrations/001_create_users.sql
    - printf '+ users\n' > db/schema.diff
    - printf 'dry run ok\n' > db/dry-run.log
    - printf 'drop table users;\n' > db/rollback.sql
    - printf 'insert into users values (1);\n' > db/seeds/users.sql
    - "printf '{\"migrations\":[\"db/migrations/001_create_users.sql\"],\"schema_diff\":\"db/schema.diff\",\"dry_run\":\"db/dry-run.log\",\"rollback_supported\":true,\"rollback_plan\":\"db/rollback.sql\",\"seed_files\":[\"db/seeds/users.sql\"]}\n' > db/migration.json"
scope:
  allow:
    - db/**
verify:
  profile: db_migration
  commands:
    - test -f db/dry-run.log
    - test -f db/rollback.sql
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 6
    max_lines_added: 12
    max_lines_deleted: 0
"#,
    )?;
    let outcome = transaction::run(repo.path(), &spec, false)?;

    assert!(matches!(outcome.status, TransactionStatus::Committed));
    let verifier = fs::read_to_string(outcome.report_path.with_file_name("verifier.json"))?;
    assert!(verifier.contains("db_migration"));
    assert!(verifier.contains("db_dry_run_present"));
    Ok(())
}

fn research_spec() -> &'static str {
    r#"
task:
  id: research_brief
  type: research.command
workspace:
  type: research.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p research
    - "printf '[{\"id\":\"s1\",\"title\":\"Source\",\"url\":\"https://example.test\"}]\\n' > research/sources.json"
    - "printf '[{\"id\":\"c1\",\"text\":\"Claim\",\"citations\":[\"s1\"]}]\\n' > research/claims.json"
    - "printf '{\"nodes\":[{\"id\":\"c1\",\"kind\":\"claim\"}],\"edges\":[]}\\n' > research/graph.json"
    - printf 'Report cites [s1].\n' > research/report.md
    - printf 'Critic reviewed c1.\n' > research/critic.md
scope:
  allow:
    - research/**
verify:
  profile: research_report
  commands:
    - test -f research/report.md
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 6
    max_lines_added: 20
    max_lines_deleted: 0
"#
}

struct OpenAiStub {
    endpoint: String,
    requests: std::sync::mpsc::Receiver<String>,
}

impl OpenAiStub {
    fn start(content: &str) -> Result<Self> {
        Self::start_bodies(vec![Self::content_body(content)])
    }

    fn content_body(content: &str) -> String {
        let escaped_content = serde_json::to_string(content).expect("serialize content");
        format!(
            r#"{{"choices":[{{"message":{{"content":{escaped_content}}}}}],"usage":{{"completion_tokens":12}}}}"#
        )
    }

    fn start_bodies(bodies: Vec<String>) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let (requests_tx, requests_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for completion_body in bodies {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let request = read_http_request(&mut stream).unwrap_or_default();
                let _ = requests_tx.send(request);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    completion_body.len(),
                    completion_body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
                let _ = stream.shutdown(Shutdown::Write);
                drain_client_close(&mut stream);
            }
        });
        Ok(Self {
            endpoint,
            requests: requests_rx,
        })
    }

    fn received_request(&self) -> Result<String> {
        Ok(self.requests.recv_timeout(Duration::from_secs(2))?)
    }
}

fn with_deepseek_stub_env<T>(endpoint: &str, run: impl FnOnce() -> Result<T>) -> Result<T> {
    static ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    let _guard = ENV_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .expect("deepseek stub env mutex");
    let previous_base = std::env::var_os("DEEPSEEK_API_BASE_URL");
    let previous_base_short = std::env::var_os("DEEPSEEK_BASE_URL");
    let previous_key = std::env::var_os("DEEPSEEK_API_KEY");
    let previous_key_file = std::env::var_os("DEEPSEEK_API_KEY_FILE");
    let previous_anthropic = std::env::var_os("ANTHROPIC_AUTH_TOKEN");
    let previous_anthropic_file = std::env::var_os("ANTHROPIC_AUTH_TOKEN_FILE");
    let previous_model = std::env::var_os("DEEPSEEK_MODEL");
    std::env::set_var("DEEPSEEK_API_BASE_URL", endpoint);
    std::env::remove_var("DEEPSEEK_BASE_URL");
    std::env::set_var("DEEPSEEK_API_KEY", "test-key");
    std::env::remove_var("DEEPSEEK_API_KEY_FILE");
    std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
    std::env::remove_var("ANTHROPIC_AUTH_TOKEN_FILE");
    std::env::set_var("DEEPSEEK_MODEL", "deepseek-test");
    let result = run();
    restore_env("DEEPSEEK_API_BASE_URL", previous_base);
    restore_env("DEEPSEEK_BASE_URL", previous_base_short);
    restore_env("DEEPSEEK_API_KEY", previous_key);
    restore_env("DEEPSEEK_API_KEY_FILE", previous_key_file);
    restore_env("ANTHROPIC_AUTH_TOKEN", previous_anthropic);
    restore_env("ANTHROPIC_AUTH_TOKEN_FILE", previous_anthropic_file);
    restore_env("DEEPSEEK_MODEL", previous_model);
    result
}

fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn read_http_request(stream: &mut std::net::TcpStream) -> std::io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 512];
    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Ok(String::from_utf8_lossy(&buffer).to_string());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(header_end) = find_header_end(&buffer) {
            let headers = String::from_utf8_lossy(&buffer[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            let body_start = header_end + 4;
            while buffer.len().saturating_sub(body_start) < content_length {
                let read = stream.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            return Ok(String::from_utf8_lossy(&buffer).to_string());
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn drain_client_close(stream: &mut impl Read) {
    let mut chunk = [0_u8; 128];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return,
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                return;
            }
            Err(_) => return,
        }
    }
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

fn shell_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}
