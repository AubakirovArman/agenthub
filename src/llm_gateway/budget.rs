use serde_json::Value;

use crate::llm_gateway::types::{BudgetDecision, BudgetPolicy, GatewaySummary};

pub fn policy_from_context(context_pack: &Value) -> BudgetPolicy {
    BudgetPolicy {
        max_tx_cost_usd: context_budget(context_pack).or_else(env_budget),
        max_daily_cost_usd: env_f64("AGENTHUB_MAX_DAILY_COST_USD"),
        prefer_local_under_complexity: env_f64("AGENTHUB_PREFER_LOCAL_UNDER_COMPLEXITY"),
    }
}

pub fn evaluate(context_pack: &Value, summary: &GatewaySummary) -> BudgetDecision {
    let policy = policy_from_context(context_pack);
    let blocked = policy
        .max_tx_cost_usd
        .filter(|max| summary.total_cost_usd > *max);
    BudgetDecision {
        allowed: blocked.is_none(),
        estimated_tx_cost_usd: summary.total_cost_usd,
        max_tx_cost_usd: policy.max_tx_cost_usd,
        reason: blocked.map(|max| {
            format!(
                "estimated transaction model cost {:.6} exceeds max_tx_cost_usd {:.6}",
                summary.total_cost_usd, max
            )
        }),
    }
}

fn context_budget(context_pack: &Value) -> Option<f64> {
    context_pack
        .get("agent_spec")?
        .get("topology")?
        .get("routing")?
        .get("max_estimated_cost_usd")?
        .as_f64()
}

fn env_budget() -> Option<f64> {
    env_f64("AGENTHUB_MAX_TX_COST_USD")
}

fn env_f64(name: &str) -> Option<f64> {
    std::env::var(name).ok()?.parse().ok()
}
