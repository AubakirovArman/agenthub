use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{json, Value};

use super::is_active_truth;
use super::storage::read_records;
use super::MemoryRecord;
use super::{memory_paths, MemoryPaths};

pub(super) fn compact_project_state(root: &Path) -> Result<()> {
    let paths = memory_paths(root)?;
    let records = read_records(&paths.memory.join("committed.jsonl"))?;
    let recent_workspace_changes = records
        .iter()
        .filter(|record| record.kind.ends_with("_change"))
        .rev()
        .take(20)
        .map(|record| {
            json!({
                "id": record.id,
                "type": record.kind,
                "tx_id": record.tx_id,
                "task_id": record.task_id,
                "domain": record.content.get("domain").cloned().unwrap_or_else(|| json!("unknown")),
                "changed_files": record.content.get("changed_files").cloned().unwrap_or_else(|| json!([])),
                "created_at": record.created_at,
            })
        })
        .collect::<Vec<_>>();

    let compacted = json!({
        "updated_at": Utc::now(),
        "records": records.len(),
        "recent_workspace_changes": recent_workspace_changes,
    });
    let path = paths.memory.join("compacted/project_state.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    write_json(&path, &compacted)?;
    write_views(root, &records)
}

pub(super) fn write_views(root: &Path, records: &[MemoryRecord]) -> Result<()> {
    let paths = memory_paths(root)?;
    let views = paths.memory.join("views");
    fs::create_dir_all(&views).with_context(|| format!("create {}", views.display()))?;

    write_json(&views.join("project_state.json"), &project_state(records))?;
    write_json(
        &views.join("code_architecture.json"),
        &records_by_kind(
            records,
            &["architecture_decision", "component", "api_endpoint"],
        ),
    )?;
    write_json(
        &views.join("current_routes.json"),
        &records_by_kind(records, &["route"]),
    )?;
    write_json(
        &views.join("dependency_policy.json"),
        &records_by_kind(records, &["dependency_policy", "forbidden_library"]),
    )?;
    write_json(&views.join("known_failures.json"), &known_failures(&paths)?)?;
    write_json(&paths.memory.join("audit.json"), &audit(&paths, records)?)?;
    Ok(())
}

fn project_state(records: &[MemoryRecord]) -> Value {
    json!({
        "updated_at": Utc::now(),
        "active_records": records.iter().filter(|record| is_active(record)).count(),
        "domains": domains(records),
        "recent": records.iter().rev().take(20).collect::<Vec<_>>(),
    })
}

fn records_by_kind(records: &[MemoryRecord], kinds: &[&str]) -> Value {
    json!({
        "updated_at": Utc::now(),
        "records": records
            .iter()
            .filter(|record| is_active(record) && kinds.contains(&record.kind.as_str()))
            .collect::<Vec<_>>(),
    })
}

fn known_failures(paths: &MemoryPaths) -> Result<Value> {
    let failed = read_records(&paths.memory.join("failed_attempts.jsonl"))?;
    Ok(json!({
        "updated_at": Utc::now(),
        "warning_only": true,
        "records": failed,
    }))
}

fn audit(paths: &MemoryPaths, records: &[MemoryRecord]) -> Result<Value> {
    let failed = read_records(&paths.memory.join("failed_attempts.jsonl"))?;
    Ok(json!({
        "updated_at": Utc::now(),
        "active": records.iter().filter(|record| is_active(record)).count(),
        "superseded": records.iter().filter(|record| record.status.as_deref() == Some("superseded")).count(),
        "stale": records.iter().filter(|record| record.stale).count(),
        "failed_attempt_warnings": failed.len(),
        "contradictions": [],
    }))
}

fn domains(records: &[MemoryRecord]) -> Vec<String> {
    let mut domains = records
        .iter()
        .filter_map(|record| record.schema.as_deref())
        .filter_map(|schema| schema.split('.').next())
        .map(str::to_string)
        .collect::<Vec<_>>();
    domains.sort();
    domains.dedup();
    domains
}

fn is_active(record: &MemoryRecord) -> bool {
    is_active_truth(record, Utc::now())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}
