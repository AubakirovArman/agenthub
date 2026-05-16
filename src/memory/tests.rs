use anyhow::Result;
use serde_json::json;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::agent_dir;

use super::{
    build_summary, failed_attempt_warnings, inspect, record_failed_attempt, retrieve_relevant,
    retrieve_relevant_scored, run_audit, write_typed_fact, TypedMemoryInput,
};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn typed_memory_write_retrieval_and_views() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;

    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "architecture_decision".to_string(),
            domain: "code".to_string(),
            content: json!({ "domain": "code", "decision": "Use runtime traits" }),
            task_id: Some("task-1".to_string()),
            supersedes: None,
            confidence: Some(0.9),
        },
    )?;
    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "route".to_string(),
            domain: "code".to_string(),
            content: json!({ "domain": "code", "path": "/courses" }),
            task_id: Some("task-1".to_string()),
            supersedes: None,
            confidence: None,
        },
    )?;

    let records = retrieve_relevant(dir.path(), "code", 10)?;
    assert!(records
        .iter()
        .any(|record| record.kind == "architecture_decision"));
    assert!(records.iter().any(|record| record.kind == "route"));
    assert!(dir.path().join(".agent/schemas/core.memory.yaml").exists());
    assert!(dir.path().join(".agent/schemas/code.memory.yaml").exists());

    let architecture = std::fs::read_to_string(
        dir.path()
            .join(".agent/memory/views/code_architecture.json"),
    )?;
    let routes =
        std::fs::read_to_string(dir.path().join(".agent/memory/views/current_routes.json"))?;
    let audit = std::fs::read_to_string(dir.path().join(".agent/memory/audit.json"))?;
    assert!(architecture.contains("Use runtime traits"));
    assert!(routes.contains("/courses"));
    assert!(audit.contains("\"active\": 2"));
    Ok(())
}

#[test]
fn chat_memory_uses_global_home_without_project_runtime() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        write_typed_fact(
            root.path(),
            TypedMemoryInput {
                kind: "architecture_decision".to_string(),
                domain: "core".to_string(),
                content: json!({ "note": "Prefer terse terminal answers", "source": "test" }),
                task_id: Some("manual-memory".to_string()),
                supersedes: None,
                confidence: Some(0.9),
            },
        )?;

        assert!(!root.path().join(".agent").exists());
        let global_memory = home.path().join("memory");
        let committed = std::fs::read_to_string(global_memory.join("committed.jsonl"))?;
        assert!(committed.contains("Prefer terse terminal answers"));

        let stats = inspect(root.path())?;
        assert_eq!(stats.committed, 1);
        assert_eq!(stats.failed_attempts, 0);

        let summary = build_summary(root.path())?;
        assert!(summary
            .active_decisions
            .iter()
            .any(|item| item.contains("architecture_decision")));

        let scored = retrieve_relevant_scored(root.path(), "code", 10)?;
        assert!(scored
            .iter()
            .any(|item| item.record.content["note"] == "Prefer terse terminal answers"));

        let audit = run_audit(root.path())?;
        assert_eq!(audit.active, 1);
        assert!(global_memory.join("audit.json").exists());
        assert!(!root.path().join(".agent").exists());
        Ok(())
    })
}

#[test]
fn failed_attempts_are_warning_memory_not_truth() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;

    record_failed_attempt(dir.path(), "tx-1", "blocked_task", "missing SECRET_TOKEN")?;

    let records = retrieve_relevant(dir.path(), "code", 10)?;
    assert!(records.is_empty());
    let committed = std::fs::read_to_string(dir.path().join(".agent/memory/committed.jsonl"))?;
    let known =
        std::fs::read_to_string(dir.path().join(".agent/memory/views/known_failures.json"))?;
    assert!(!committed.contains("blocked_task"));
    assert!(known.contains("\"warning_only\": true"));
    assert!(known.contains("blocked_task"));
    Ok(())
}

#[test]
fn memory_summary_audit_scoring_and_warnings_are_actionable() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )?;

    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "architecture_decision".to_string(),
            domain: "code".to_string(),
            content: json!({
                "domain": "code",
                "topic": "license",
                "decision": "Use Apache-2.0"
            }),
            task_id: Some("task-license".to_string()),
            supersedes: None,
            confidence: Some(0.95),
        },
    )?;
    record_failed_attempt(
        dir.path(),
        "tx-rollback",
        "package_lock_change",
        "package lock changed without approval",
    )?;

    let summary = build_summary(dir.path())?;
    assert!(summary.stack.iter().any(|item| item == "Rust CLI"));
    assert!(summary
        .active_decisions
        .iter()
        .any(|item| item.contains("Apache-2.0")));

    let scored = retrieve_relevant_scored(dir.path(), "code", 10)?;
    assert!(scored[0].score > 0.7);
    assert!(scored[0].reasons.contains(&"same_domain".to_string()));

    let warnings = failed_attempt_warnings(dir.path(), "package lock change", 10)?;
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].mitigation.contains("dependency-change"));

    let audit = run_audit(dir.path())?;
    assert_eq!(audit.active, 1);
    assert_eq!(audit.failed_attempts, 1);
    assert!(dir.path().join(".agent/memory/audit.json").exists());
    Ok(())
}

fn with_agenthub_home<T>(home: &Path, run: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock poisoned");
    let previous = std::env::var_os("AGENTHUB_HOME");
    std::env::set_var("AGENTHUB_HOME", home);
    let result = run();
    match previous {
        Some(value) => std::env::set_var("AGENTHUB_HOME", value),
        None => std::env::remove_var("AGENTHUB_HOME"),
    }
    result
}
