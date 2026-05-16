use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::memory_paths;
use super::storage::read_records;
use super::MemoryRecord;
use super::{derived_conflict_key, is_active_truth, is_expired};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAudit {
    pub active: usize,
    pub stale: usize,
    pub expired: usize,
    pub failed_attempts: usize,
    pub low_confidence: usize,
    pub missing_last_verified_commit: usize,
    pub conflicting_decisions: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn run_audit(root: &Path) -> Result<MemoryAudit> {
    let paths = memory_paths(root)?;
    let committed = read_records(&paths.memory.join("committed.jsonl"))?;
    let failed = read_records(&paths.memory.join("failed_attempts.jsonl"))?;
    let now = Utc::now();
    let audit = MemoryAudit {
        active: committed
            .iter()
            .filter(|record| is_active_truth(record, now))
            .count(),
        stale: committed.iter().filter(|record| record.stale).count(),
        expired: committed
            .iter()
            .filter(|record| is_expired(record, now))
            .count(),
        failed_attempts: failed.len(),
        low_confidence: committed
            .iter()
            .filter(|record| record.confidence.unwrap_or(1.0) < 0.5)
            .count(),
        missing_last_verified_commit: committed
            .iter()
            .filter(|record| is_active_truth(record, now) && record.last_verified_commit.is_none())
            .count(),
        conflicting_decisions: conflicting_decisions(&committed, now),
        warnings: Vec::new(),
    }
    .with_warnings();
    let path = paths.memory.join("audit.json");
    fs::write(&path, serde_json::to_string_pretty(&audit)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(audit)
}

impl MemoryAudit {
    fn with_warnings(mut self) -> Self {
        if self.stale > 0 {
            self.warnings
                .push(format!("{} stale records should be reviewed.", self.stale));
        }
        if self.expired > 0 {
            self.warnings.push(format!(
                "{} expired records are excluded from context.",
                self.expired
            ));
        }
        if self.low_confidence > 0 {
            self.warnings.push(format!(
                "{} records have confidence below 0.5.",
                self.low_confidence
            ));
        }
        if self.missing_last_verified_commit > 0 {
            self.warnings.push(format!(
                "{} active records have no last_verified_commit.",
                self.missing_last_verified_commit
            ));
        }
        if !self.conflicting_decisions.is_empty() {
            self.warnings.push(format!(
                "{} possible decision conflicts detected.",
                self.conflicting_decisions.len()
            ));
        }
        self
    }
}

fn conflicting_decisions(records: &[MemoryRecord], now: DateTime<Utc>) -> Vec<String> {
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for record in records
        .iter()
        .filter(|record| is_active_truth(record, now) && is_decision_kind(&record.kind))
    {
        let decision = decision_value(record);
        groups
            .entry(derived_conflict_key(record).unwrap_or_else(|| record.kind.clone()))
            .or_default()
            .insert(decision);
    }
    groups
        .into_iter()
        .filter(|(_, values)| values.len() > 1)
        .map(|(topic, values)| {
            format!(
                "{topic} => {}",
                values.into_iter().collect::<Vec<_>>().join(" | ")
            )
        })
        .collect()
}

fn decision_value(record: &MemoryRecord) -> String {
    for key in ["decision", "policy", "rule", "summary"] {
        if let Some(value) = record.content.get(key).and_then(Value::as_str) {
            return value.to_string();
        }
    }
    record.id.clone()
}

fn is_decision_kind(kind: &str) -> bool {
    matches!(
        kind,
        "architecture_decision" | "dependency_policy" | "test_policy" | "style_rule"
    )
}
