use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::llm_gateway::costs;
use crate::llm_gateway::types::{GatewayArtifacts, GatewaySummary, ModelCallMetadata};
use crate::observability::{redact_value, sha256_json, write_jsonl, write_pretty_json};

pub fn write_gateway_artifacts(
    tx_dir: &Path,
    context_pack: &Value,
    context_hash: &str,
) -> Result<GatewayArtifacts> {
    let calls = planned_calls(context_pack, context_hash)?;
    let summary = GatewaySummary {
        redaction_enabled: true,
        raw_trace_enabled: raw_traces_enabled(),
        model_call_count: calls.len(),
        total_tokens: calls.iter().map(|call| call.total_tokens).sum(),
        total_cost_usd: calls.iter().map(|call| call.estimated_cost_usd).sum(),
    };

    write_pretty_json(&tx_dir.join("model_call_metadata.json"), &calls)?;
    write_pretty_json(&tx_dir.join("llm_gateway_summary.json"), &summary)?;
    for call in &calls {
        write_trace_record(tx_dir, call)?;
    }

    Ok(GatewayArtifacts {
        model_calls: calls,
        summary,
    })
}

fn planned_calls(context_pack: &Value, context_hash: &str) -> Result<Vec<ModelCallMetadata>> {
    let mut calls = Vec::new();
    if let Some(routes) = context_pack.get("agent_routes") {
        if let Some(role_routes) = routes.get("roles").and_then(Value::as_array) {
            for route in role_routes {
                push_route(&mut calls, Some(route), "agent", context_pack, context_hash)?;
            }
            return Ok(calls);
        }
        push_route(
            &mut calls,
            routes.get("executor"),
            "executor",
            context_pack,
            context_hash,
        )?;
        push_route(
            &mut calls,
            routes.get("reviewer"),
            "reviewer",
            context_pack,
            context_hash,
        )?;
        push_route(
            &mut calls,
            routes.get("repair"),
            "repair",
            context_pack,
            context_hash,
        )?;
    }
    Ok(calls)
}

fn push_route(
    calls: &mut Vec<ModelCallMetadata>,
    route: Option<&Value>,
    fallback_role: &str,
    context_pack: &Value,
    context_hash: &str,
) -> Result<()> {
    let Some(route) = route.and_then(|value| if value.is_null() { None } else { Some(value) })
    else {
        return Ok(());
    };
    let role = route
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or(fallback_role);
    let selected = route
        .get("selected_adapter")
        .and_then(Value::as_str)
        .unwrap_or("command");
    let prompt_hash = prompt_hash(role, context_pack)?;
    let prompt_tokens = estimate_tokens(context_pack);
    let price = costs::estimate(selected, prompt_tokens, 0);
    let model = route
        .get("model")
        .and_then(Value::as_str)
        .map(str::to_string);
    let private_model = model
        .as_deref()
        .map(|model| {
            private_models(context_pack)
                .iter()
                .any(|item| item == model)
        })
        .unwrap_or(false);
    let runner = selected_runner(context_pack, private_model);

    calls.push(ModelCallMetadata {
        id: format!("call-{}", &Uuid::new_v4().to_string()[..8]),
        role: role.to_string(),
        requested_adapter: route_string(route, "requested_adapter", selected),
        selected_adapter: selected.to_string(),
        model,
        private_model,
        runner,
        routing_policy: if private_model {
            "private_model".to_string()
        } else {
            "default".to_string()
        },
        status: "planned".to_string(),
        context_pack_hash: context_hash.to_string(),
        prompt_hash,
        prompt_tokens,
        completion_tokens: 0,
        total_tokens: prompt_tokens,
        estimated_cost_usd: price.cost_usd,
        pricing_source: price.source,
        latency_ms: None,
        error: None,
        created_at: Utc::now(),
    });
    Ok(())
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

fn prompt_hash(role: &str, context_pack: &Value) -> Result<String> {
    sha256_json(&json!({
        "role": role,
        "task": context_pack.get("agent_spec").and_then(|spec| spec.get("task")),
        "skills": context_pack.get("skills"),
        "memory": context_pack.get("memory"),
    }))
}

fn route_string(route: &Value, field: &str, fallback: &str) -> String {
    route
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn private_models(context_pack: &Value) -> Vec<String> {
    context_pack
        .get("enterprise")
        .and_then(|enterprise| enterprise.get("private_models"))
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn selected_runner(context_pack: &Value, private_model: bool) -> Option<String> {
    let enterprise = context_pack.get("enterprise")?;
    if private_model {
        if let Some(runner) = enterprise.get("private_runner").and_then(Value::as_str) {
            return Some(runner.to_string());
        }
    }
    enterprise
        .get("runner_default")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn estimate_tokens(value: &Value) -> usize {
    serde_json::to_string(value)
        .map(|text| (text.len() / 4).max(1))
        .unwrap_or(0)
}

fn raw_traces_enabled() -> bool {
    std::env::var("AGENTHUB_RAW_TRACES").ok().as_deref() == Some("1")
}
