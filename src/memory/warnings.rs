use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::memory_paths;
use super::storage::read_records;
use super::MemoryRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedAttemptWarning {
    pub id: String,
    pub tx_id: String,
    pub task_id: Option<String>,
    pub reason: String,
    pub score: f32,
    pub mitigation: String,
    pub reasons: Vec<String>,
}

pub fn failed_attempt_warnings(
    root: &Path,
    query: &str,
    limit: usize,
) -> Result<Vec<FailedAttemptWarning>> {
    let paths = memory_paths(root)?;
    let query_terms = terms(query);
    let mut warnings = read_records(&paths.memory.join("failed_attempts.jsonl"))?
        .into_iter()
        .filter_map(|record| score_warning(record, &query_terms))
        .filter(|warning| warning.score >= 0.35)
        .collect::<Vec<_>>();

    warnings.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.tx_id.cmp(&a.tx_id))
    });
    warnings.truncate(limit);
    Ok(warnings)
}

fn score_warning(
    record: MemoryRecord,
    query_terms: &HashSet<String>,
) -> Option<FailedAttemptWarning> {
    let reason = record
        .content
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("previous failed attempt")
        .to_string();
    let mut score: f32 = 0.2;
    let mut reasons = vec!["previous_failed_attempt".to_string()];
    let text = format!("{} {}", record.task_id.clone().unwrap_or_default(), reason);
    let overlap = terms(&text).intersection(query_terms).count();

    if overlap > 0 {
        score += (overlap as f32 * 0.15).min(0.45);
        reasons.push("same_task_terms".to_string());
    }
    if record.status.as_deref() == Some("warning") {
        score += 0.1;
        reasons.push("warning_only_memory".to_string());
    }

    Some(FailedAttemptWarning {
        id: record.id,
        tx_id: record.tx_id,
        task_id: record.task_id,
        reason: reason.clone(),
        score: score.min(1.0),
        mitigation: mitigation_for(&reason),
        reasons,
    })
}

fn terms(text: &str) -> HashSet<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|term| term.len() >= 4)
        .map(str::to_lowercase)
        .collect()
}

fn mitigation_for(reason: &str) -> String {
    let lower = reason.to_lowercase();
    if lower.contains("lock") || lower.contains("package") {
        "Use an explicit dependency-change skill or approval before package changes.".to_string()
    } else if lower.contains("env") || lower.contains("secret") || lower.contains("token") {
        "Resolve the missing environment or secret, then resume the transaction.".to_string()
    } else if lower.contains("diff") || lower.contains("scope") {
        "Narrow the task or extend scope.allow before retrying.".to_string()
    } else if lower.contains("verifier") || lower.contains("test") {
        "Run the verifier locally and fix the failing check before retrying.".to_string()
    } else {
        "Inspect the previous report and adjust the plan before retrying.".to_string()
    }
}
