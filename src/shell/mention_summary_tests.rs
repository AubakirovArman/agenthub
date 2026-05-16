use std::fs;

use anyhow::Result;
use serde_json::json;

use crate::{agent_dir, memory, memory::TypedMemoryInput};

use super::mention_summary;

#[test]
fn resolves_transaction_and_memory_mentions() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx_dir = dir.path().join(".agent/tx/tx-demo");
    fs::create_dir_all(&tx_dir)?;
    fs::write(
        tx_dir.join("report.md"),
        "# AgentHub Report\n\n- Status: `COMMITTED`\n\nChanged files: 1\n",
    )?;
    memory::write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "dependency_policy".to_string(),
            domain: "code".to_string(),
            content: json!({"rule": "Use fetch instead of axios"}),
            task_id: Some("docs".to_string()),
            supersedes: None,
            confidence: Some(0.9),
            ttl_days: None,
            pinned: false,
            conflict_key: None,
        },
    )?;

    let tx = mention_summary::resolve(dir.path(), "tx:tx-demo", "")?;
    assert!(tx.contains("@tx `tx-demo` status `COMMITTED`"));
    let memory = mention_summary::resolve(dir.path(), "memory:fetch", "")?;
    assert!(memory.contains("@memory `fetch`"));
    assert!(memory.contains("Use fetch instead of axios"));
    Ok(())
}
