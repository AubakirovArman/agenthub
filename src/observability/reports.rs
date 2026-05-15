use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde_json::json;

use super::{write_jsonl, write_pretty_json, RedactionFinding};

pub fn write_secret_scan_record(
    tx_dir: &Path,
    source: &str,
    findings: &[RedactionFinding],
) -> Result<()> {
    if findings.is_empty() {
        return Ok(());
    }
    write_jsonl(
        &tx_dir.join("secret_scan.jsonl"),
        &json!({
            "ts": Utc::now(),
            "source": source,
            "findings": findings,
            "redacted": true
        }),
    )
}

pub fn write_redaction_report(
    tx_dir: &Path,
    source: &str,
    findings: &[RedactionFinding],
    raw_requested: bool,
    raw_allowed: bool,
) -> Result<()> {
    write_pretty_json(
        &tx_dir.join("redaction_report.json"),
        &json!({
            "source": source,
            "scanned_at": Utc::now(),
            "findings": findings,
            "redacted": !findings.is_empty(),
            "raw_trace_requested": raw_requested,
            "raw_trace_written": raw_allowed,
            "raw_trace_policy": raw_policy(raw_requested, raw_allowed, findings),
        }),
    )
}

fn raw_policy(raw_requested: bool, raw_allowed: bool, findings: &[RedactionFinding]) -> String {
    match (raw_requested, raw_allowed, findings.is_empty()) {
        (false, _, _) => "raw_traces_disabled".to_string(),
        (true, true, _) => "raw_traces_allowed".to_string(),
        (true, false, false) => "raw_traces_blocked_secret_findings".to_string(),
        (true, false, true) => "raw_traces_blocked".to_string(),
    }
}
