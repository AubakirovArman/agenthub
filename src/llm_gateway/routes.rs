use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::llm_gateway::costs;
use crate::llm_gateway::types::ModelCallMetadata;
use crate::observability::sha256_json;

pub(super) fn planned_calls(
    context_pack: &Value,
    context_hash: &str,
) -> Result<Vec<ModelCallMetadata>> {
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

    calls.push(ModelCallMetadata {
        id: format!("call-{}", &Uuid::new_v4().to_string()[..8]),
        role: role.to_string(),
        requested_adapter: route_string(route, "requested_adapter", selected),
        selected_adapter: selected.to_string(),
        model,
        private_model,
        runner: selected_runner(context_pack, private_model),
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
