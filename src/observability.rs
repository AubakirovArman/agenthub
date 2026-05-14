use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityArtifacts {
    pub context_pack_trace: ContextPackTrace,
    pub cost_profile: CostProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPackTrace {
    pub context_pack_hash: String,
    pub memory_ids: Vec<String>,
    pub skill_ids: Vec<String>,
    pub file_refs: Vec<String>,
    pub policy_rules: Vec<String>,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProfile {
    pub currency: String,
    pub total_usd: f64,
    pub estimated_tokens: usize,
    pub breakdown: Vec<CostItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostItem {
    pub label: String,
    pub estimated_tokens: usize,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFingerprint {
    pub fingerprint: String,
    pub tx_id: String,
    pub task_id: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

pub fn write_start_artifacts(
    tx_dir: &Path,
    context_pack: &Value,
    skill_ids: &[String],
    memory_ids: &[String],
) -> Result<ObservabilityArtifacts> {
    let estimated_tokens = estimate_tokens(context_pack);
    let trace = ContextPackTrace {
        context_pack_hash: sha256_json(context_pack)?,
        memory_ids: memory_ids.to_vec(),
        skill_ids: skill_ids.to_vec(),
        file_refs: Vec::new(),
        policy_rules: vec!["least_context".to_string(), "scope_only".to_string()],
        estimated_tokens,
    };
    let cost_profile = CostProfile {
        currency: "USD".to_string(),
        total_usd: 0.0,
        estimated_tokens,
        breakdown: vec![
            CostItem {
                label: "Intent Normalization".to_string(),
                estimated_tokens: 0,
                cost_usd: 0.0,
            },
            CostItem {
                label: "Context Pack Build".to_string(),
                estimated_tokens,
                cost_usd: 0.0,
            },
            CostItem {
                label: "Agent Execution".to_string(),
                estimated_tokens: 0,
                cost_usd: 0.0,
            },
        ],
    };

    write_json(tx_dir.join("context_pack_trace.json").as_path(), &trace)?;
    write_json(tx_dir.join("cost.json").as_path(), &cost_profile)?;
    write_json(
        tx_dir.join("skill_trace.json").as_path(),
        &json!({
            "active_skills": skill_ids,
            "loaded_at": Utc::now(),
        }),
    )?;
    append_redacted_trace(
        tx_dir,
        &json!({
            "type": "llm_gateway",
            "event": "no_model_calls",
            "redaction": "enabled",
            "created_at": Utc::now(),
        }),
    )?;

    Ok(ObservabilityArtifacts {
        context_pack_trace: trace,
        cost_profile,
    })
}

pub fn write_error_fingerprint(
    tx_dir: &Path,
    tx_id: &str,
    task_id: &str,
    reason: &str,
) -> Result<ErrorFingerprint> {
    let fingerprint = format!(
        "{}_{}",
        normalize_reason(reason),
        sha256_short(reason.as_bytes())
    );
    let event = ErrorFingerprint {
        fingerprint,
        tx_id: tx_id.to_string(),
        task_id: task_id.to_string(),
        reason: redact_text(reason)?,
        created_at: Utc::now(),
    };
    write_json(tx_dir.join("error_fingerprint.json").as_path(), &event)?;
    Ok(event)
}

pub fn redact_text(input: &str) -> Result<String> {
    let replacements = [
        (
            r#"(?i)(api[_-]?key|token|password|secret|database_url|db_url)\s*[:=]\s*['"]?[^'"\s]+"#,
            "$1=<redacted>",
        ),
        (r#"(?i)bearer\s+[A-Za-z0-9._\-]+"#, "Bearer <redacted>"),
        (r#"sk-[A-Za-z0-9_\-]{10,}"#, "sk-<redacted>"),
        (
            r#"(?i)(postgres|postgresql|mysql|mongodb|redis)://[^'"\s]+"#,
            "$1://<redacted>",
        ),
    ];

    let mut output = input.to_string();
    for (pattern, replacement) in replacements {
        let regex = Regex::new(pattern)?;
        output = regex.replace_all(&output, replacement).to_string();
    }
    Ok(output)
}

fn append_redacted_trace(tx_dir: &Path, event: &Value) -> Result<()> {
    append_jsonl(&tx_dir.join("redacted_api.jsonl"), event)?;

    if env::var("AGENTHUB_RAW_TRACES").ok().as_deref() == Some("1") {
        append_jsonl(&tx_dir.join("raw_api.jsonl"), event)?;
    }

    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

fn append_jsonl(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn sha256_json(value: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_short(bytes: &[u8]) -> String {
    sha256_hex(bytes)[..12].to_string()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn estimate_tokens(value: &Value) -> usize {
    serde_json::to_string(value)
        .map(|text| (text.len() / 4).max(1))
        .unwrap_or(0)
}

fn normalize_reason(reason: &str) -> String {
    let mut normalized = reason
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    while normalized.contains("__") {
        normalized = normalized.replace("__", "_");
    }
    normalized.trim_matches('_').chars().take(40).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_common_secret_shapes() -> Result<()> {
        let text = "token=abcd1234 Bearer secret.jwt.value postgres://user:pass@localhost/db sk-1234567890abcdef";
        let redacted = redact_text(text)?;

        assert!(!redacted.contains("abcd1234"));
        assert!(!redacted.contains("secret.jwt.value"));
        assert!(!redacted.contains("user:pass"));
        assert!(!redacted.contains("1234567890abcdef"));
        assert!(redacted.contains("<redacted>"));
        Ok(())
    }
}
