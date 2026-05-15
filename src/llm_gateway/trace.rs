use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::llm_gateway::types::{GatewayArtifacts, GatewaySummary, ModelCallMetadata};
use crate::llm_gateway::{budget, planning, routes};
use crate::observability::{redact_value, write_jsonl, write_pretty_json};

pub fn write_gateway_artifacts(
    tx_dir: &Path,
    context_pack: &Value,
    context_hash: &str,
) -> Result<GatewayArtifacts> {
    let calls = routes::planned_calls(context_pack, context_hash)?;
    let summary = GatewaySummary {
        redaction_enabled: true,
        raw_trace_enabled: raw_traces_enabled(),
        model_call_count: calls.len(),
        total_tokens: calls.iter().map(|call| call.total_tokens).sum(),
        total_cost_usd: calls.iter().map(|call| call.estimated_cost_usd).sum(),
    };
    let provider_plan = planning::build_provider_plan(&calls);
    let budget = budget::evaluate(context_pack, &summary);

    write_pretty_json(&tx_dir.join("model_call_metadata.json"), &calls)?;
    write_pretty_json(&tx_dir.join("llm_provider_plan.json"), &provider_plan)?;
    write_pretty_json(&tx_dir.join("llm_budget.json"), &budget)?;
    write_pretty_json(&tx_dir.join("llm_gateway_summary.json"), &summary)?;
    for call in &calls {
        write_trace_record(tx_dir, call)?;
    }
    if !budget.allowed {
        return Err(anyhow!(
            "{}",
            budget
                .reason
                .as_deref()
                .unwrap_or("LLM gateway budget rejected transaction")
        ));
    }

    Ok(GatewayArtifacts {
        model_calls: calls,
        summary,
        provider_plan,
        budget,
    })
}

fn write_trace_record(tx_dir: &Path, call: &ModelCallMetadata) -> Result<()> {
    let raw = json!({ "type": "llm_gateway", "event": "model_call_metadata", "call": call });
    let redacted = redact_value(&raw)?;
    write_jsonl(&tx_dir.join("redacted_api.jsonl"), &redacted)?;
    if raw_traces_enabled() {
        write_jsonl(&tx_dir.join("raw_api.jsonl"), &raw)?;
    }
    Ok(())
}

fn raw_traces_enabled() -> bool {
    std::env::var("AGENTHUB_RAW_TRACES").ok().as_deref() == Some("1")
}
