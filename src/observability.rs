mod hash;
mod redaction;
mod reports;
mod storage;
mod tokens;

use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::llm_gateway::{self, GatewaySummary};

use hash::normalize_reason;
pub use hash::{sha256_json, sha256_short};
pub use redaction::{
    merge_findings, redact_file_in_place, redact_text, redact_text_with_findings, redact_value,
    redact_value_with_findings, RedactionFinding,
};
pub use reports::write_secret_scan_record;
pub use storage::{append_jsonl as write_jsonl, write_json as write_pretty_json};
use tokens::estimate_tokens;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityArtifacts {
    pub context_pack_trace: ContextPackTrace,
    pub cost_profile: CostProfile,
    pub gateway_summary: GatewaySummary,
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

pub fn write_context_pack_artifacts(tx_dir: &Path, context: &Value) -> Result<Value> {
    let (redacted, findings) = redact_value_with_findings(context)?;
    let raw_requested = raw_traces_requested();
    let raw_allowed = raw_requested && (findings.is_empty() || raw_secret_traces_allowed());
    reports::write_redaction_report(
        tx_dir,
        "context_pack",
        &findings,
        raw_requested,
        raw_allowed,
    )?;
    write_pretty_json(&tx_dir.join("context_pack.json"), &redacted)?;
    if raw_allowed {
        write_pretty_json(&tx_dir.join("raw_context_pack.json"), context)?;
    }
    Ok(redacted)
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
    let gateway =
        llm_gateway::write_gateway_artifacts(tx_dir, context_pack, &trace.context_pack_hash)?;
    let cost_profile = build_cost_profile(estimated_tokens, &gateway.summary);

    write_pretty_json(&tx_dir.join("context_pack_trace.json"), &trace)?;
    write_pretty_json(&tx_dir.join("cost.json"), &cost_profile)?;
    write_pretty_json(
        &tx_dir.join("skill_trace.json"),
        &json!({
            "active_skills": skill_ids,
            "loaded_at": Utc::now(),
        }),
    )?;

    Ok(ObservabilityArtifacts {
        context_pack_trace: trace,
        cost_profile,
        gateway_summary: gateway.summary,
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
    write_pretty_json(&tx_dir.join("error_fingerprint.json"), &event)?;
    Ok(event)
}

fn build_cost_profile(context_tokens: usize, gateway: &GatewaySummary) -> CostProfile {
    CostProfile {
        currency: "USD".to_string(),
        total_usd: gateway.total_cost_usd,
        estimated_tokens: context_tokens + gateway.total_tokens,
        breakdown: vec![
            CostItem {
                label: "Intent Normalization".to_string(),
                estimated_tokens: 0,
                cost_usd: 0.0,
            },
            CostItem {
                label: "Context Pack Build".to_string(),
                estimated_tokens: context_tokens,
                cost_usd: 0.0,
            },
            CostItem {
                label: "LLM Gateway Planned Calls".to_string(),
                estimated_tokens: gateway.total_tokens,
                cost_usd: gateway.total_cost_usd,
            },
        ],
    }
}

fn raw_traces_requested() -> bool {
    std::env::var("AGENTHUB_RAW_TRACES").ok().as_deref() == Some("1")
}

fn raw_secret_traces_allowed() -> bool {
    std::env::var("AGENTHUB_ALLOW_RAW_SECRET_TRACES")
        .ok()
        .as_deref()
        == Some("1")
}
