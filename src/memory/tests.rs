use anyhow::Result;
use serde_json::json;

use crate::agent_dir;

use super::{record_failed_attempt, retrieve_relevant, write_typed_fact, TypedMemoryInput};

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
