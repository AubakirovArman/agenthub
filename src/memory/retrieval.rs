use std::cmp::Ordering;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::is_active_truth;
use super::memory_paths;
use super::storage::read_records;
use super::MemoryRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemoryRecord {
    #[serde(flatten)]
    pub record: MemoryRecord,
    pub score: f32,
    pub reasons: Vec<String>,
}

pub fn retrieve_relevant_scored(
    root: &Path,
    domain: &str,
    limit: usize,
) -> Result<Vec<ScoredMemoryRecord>> {
    let paths = memory_paths(root)?;
    let records = read_records(&paths.memory.join("committed.jsonl"))?;
    let now = Utc::now();
    let mut scored = records
        .into_iter()
        .filter(|record| is_active_truth(record, now))
        .map(|record| score_record(record, domain))
        .filter(|record| record.score >= 0.25)
        .collect::<Vec<_>>();

    scored.sort_by(sort_scored);
    scored.truncate(limit);
    Ok(scored)
}

fn score_record(record: MemoryRecord, domain: &str) -> ScoredMemoryRecord {
    let mut score: f32 = 0.1;
    let mut reasons = Vec::new();

    if matches_domain(&record, domain) {
        score += 0.45;
        reasons.push("same_domain".to_string());
    }
    if matches_domain(&record, "core") {
        score += 0.25;
        reasons.push("core_memory".to_string());
    }
    if is_decision_kind(&record.kind) {
        score += 0.2;
        reasons.push("active_decision".to_string());
    }
    if record.kind.ends_with("_change") {
        score += 0.1;
        reasons.push("recent_workspace_change".to_string());
    }
    if record.last_verified_commit.is_some() {
        score += 0.1;
        reasons.push("verified_commit".to_string());
    }
    if record.confidence.unwrap_or(0.0) >= 0.8 {
        score += 0.1;
        reasons.push("high_confidence".to_string());
    }

    ScoredMemoryRecord {
        record,
        score: score.min(1.0),
        reasons,
    }
}

fn matches_domain(record: &MemoryRecord, domain: &str) -> bool {
    record.schema.as_deref().is_some_and(|schema| {
        schema == format!("{domain}.memory.v1") || schema.starts_with(&format!("{domain}."))
    }) || record
        .content
        .get("domain")
        .and_then(Value::as_str)
        .is_some_and(|item| item == domain)
}

fn is_decision_kind(kind: &str) -> bool {
    matches!(
        kind,
        "architecture_decision"
            | "dependency_policy"
            | "test_policy"
            | "style_rule"
            | "known_failure"
            | "forbidden_library"
    )
}

fn sort_scored(a: &ScoredMemoryRecord, b: &ScoredMemoryRecord) -> Ordering {
    b.score
        .partial_cmp(&a.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| b.record.created_at.cmp(&a.record.created_at))
}
