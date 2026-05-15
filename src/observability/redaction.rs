use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactionFinding {
    pub kind: String,
    pub count: usize,
}

pub fn redact_text(input: &str) -> Result<String> {
    Ok(redact_text_with_findings(input)?.0)
}

pub fn redact_text_with_findings(input: &str) -> Result<(String, Vec<RedactionFinding>)> {
    let replacements = [
        (
            "named_secret",
            r#"(?i)(api[_-]?key|token|password|secret|database_url|db_url)\s*[:=]\s*['"]?[^'"\s]+"#,
            "$1=<redacted>",
        ),
        (
            "bearer_token",
            r#"(?i)bearer\s+[A-Za-z0-9._\-]+"#,
            "Bearer <redacted>",
        ),
        (
            "openai_style_key",
            r#"sk-[A-Za-z0-9_\-]{10,}"#,
            "sk-<redacted>",
        ),
        (
            "github_token",
            r#"(ghp|gho|ghu|ghs|ghr|github_pat)_[A-Za-z0-9_]{16,}"#,
            "$1_<redacted>",
        ),
        (
            "google_api_key",
            r#"AIza[0-9A-Za-z\-_]{20,}"#,
            "AIza<redacted>",
        ),
        ("aws_access_key", r#"AKIA[0-9A-Z]{16}"#, "AKIA<redacted>"),
        (
            "database_url",
            r#"(?i)(postgres|postgresql|mysql|mongodb|redis)://[^'"\s]+"#,
            "$1://<redacted>",
        ),
    ];

    let mut output = input.to_string();
    let mut counts = BTreeMap::new();
    for (kind, pattern, replacement) in replacements {
        let regex = Regex::new(pattern)?;
        let count = regex.find_iter(&output).count();
        if count > 0 {
            *counts.entry(kind.to_string()).or_insert(0) += count;
        }
        output = regex.replace_all(&output, replacement).to_string();
    }
    Ok((output, findings_from_counts(counts)))
}

pub fn redact_value(value: &Value) -> Result<Value> {
    Ok(redact_value_with_findings(value)?.0)
}

pub fn redact_value_with_findings(value: &Value) -> Result<(Value, Vec<RedactionFinding>)> {
    match value {
        Value::String(text) => {
            let (text, findings) = redact_text_with_findings(text)?;
            Ok((Value::String(text), findings))
        }
        Value::Array(items) => {
            let mut findings = Vec::new();
            let mut redacted = Vec::new();
            for item in items {
                let (value, item_findings) = redact_value_with_findings(item)?;
                findings.extend(item_findings);
                redacted.push(value);
            }
            Ok((Value::Array(redacted), merge_findings(findings)))
        }
        Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            let mut findings = Vec::new();
            for (key, value) in map {
                if is_secret_key(key) && !value.is_null() {
                    redacted.insert(key.clone(), Value::String("<redacted>".to_string()));
                    findings.push(RedactionFinding {
                        kind: format!("secret_key:{key}"),
                        count: 1,
                    });
                } else {
                    let (value, value_findings) = redact_value_with_findings(value)?;
                    findings.extend(value_findings);
                    redacted.insert(key.clone(), value);
                }
            }
            Ok((Value::Object(redacted), merge_findings(findings)))
        }
        other => Ok((other.clone(), Vec::new())),
    }
}

pub fn redact_file_in_place(path: &Path) -> Result<Vec<RedactionFinding>> {
    let tmp = path.with_extension("redacted.tmp");
    let mut input = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut output = File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
    let mut findings = Vec::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = input
            .read(&mut buffer)
            .with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&buffer[..read]);
        let (redacted, chunk_findings) = redact_text_with_findings(&chunk)?;
        findings.extend(chunk_findings);
        output
            .write_all(redacted.as_bytes())
            .with_context(|| format!("write {}", tmp.display()))?;
    }
    let findings = merge_findings(findings);
    if findings.is_empty() {
        fs::remove_file(&tmp).with_context(|| format!("remove {}", tmp.display()))?;
    } else {
        fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))?;
    }
    Ok(findings)
}

pub fn merge_findings(findings: Vec<RedactionFinding>) -> Vec<RedactionFinding> {
    let mut counts = BTreeMap::new();
    for finding in findings {
        *counts.entry(finding.kind).or_insert(0) += finding.count;
    }
    findings_from_counts(counts)
}

fn findings_from_counts(counts: BTreeMap<String, usize>) -> Vec<RedactionFinding> {
    counts
        .into_iter()
        .map(|(kind, count)| RedactionFinding { kind, count })
        .collect()
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "token",
        "password",
        "secret",
        "credential",
        "private_key",
        "database_url",
        "db_url",
        "access_key",
        "refresh_token",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests;
