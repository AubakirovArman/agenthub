use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::agent_dir::{AgentPaths, TransactionRow};
use crate::memory::MemoryStats;
use crate::plugin_registry;
use crate::web_dashboard::read::{array_len, is_failed, is_open, read_json};
use crate::web_dashboard::{
    ContextMetrics, CostMetrics, MetricsPanel, QualityMetrics, ReliabilityMetrics, TrustMetrics,
};

pub fn collect_metrics(
    project_root: &Path,
    rows: &[TransactionRow],
    memory: &MemoryStats,
) -> Result<MetricsPanel> {
    let paths = AgentPaths::new(project_root);
    let mut quality = QualityMetrics::default();
    let mut tokens = 0;
    let mut total_cost = 0.0;
    let mut cost_count = 0;
    let mut dag_nodes = 0;

    for row in rows {
        let tx_dir = paths.tx_dir(&row.id);
        let cost = read_json(&tx_dir.join("cost.json")).unwrap_or(Value::Null);
        let dag = read_json(&tx_dir.join("dag.json")).unwrap_or(Value::Null);
        tokens += cost
            .get("estimated_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        if let Some(cost) = cost.get("total_usd").and_then(Value::as_f64) {
            total_cost += cost;
            cost_count += 1;
        }
        dag_nodes += array_len(&dag, "nodes");
        collect_gate(
            &tx_dir.join("verifier.json"),
            &mut quality.verifier_total,
            &mut quality.verifier_passed,
        );
        collect_gate(
            &tx_dir.join("review.json"),
            &mut quality.review_total,
            &mut quality.review_passed,
        );
    }

    Ok(MetricsPanel {
        reliability: reliability(rows),
        context: ContextMetrics {
            memory_records: memory.committed,
            failed_attempts: memory.failed_attempts,
            estimated_tokens: tokens,
            average_dag_nodes: average(dag_nodes, rows.len()),
        },
        quality: finish_quality(quality),
        trust: trust(project_root)?,
        cost: CostMetrics {
            total_usd: total_cost,
            average_usd: average_float(total_cost, cost_count),
            estimated_tokens: tokens,
        },
        history: crate::analytics::load_summary(project_root)?,
    })
}

fn reliability(rows: &[TransactionRow]) -> ReliabilityMetrics {
    let committed = rows.iter().filter(|row| row.status == "COMMITTED").count();
    let failed = rows.iter().filter(|row| is_failed(&row.status)).count();
    let blocked = rows
        .iter()
        .filter(|row| row.status == "BLOCKED_ON_HUMAN")
        .count();
    let open = rows.iter().filter(|row| is_open(&row.status)).count();
    ReliabilityMetrics {
        committed,
        failed,
        blocked,
        open,
        success_rate: average(committed, rows.len()),
    }
}

fn collect_gate(path: &Path, total: &mut usize, passed: &mut usize) {
    let value = read_json(path).unwrap_or(Value::Null);
    if let Some(result) = value.get("passed").and_then(Value::as_bool) {
        *total += 1;
        if result {
            *passed += 1;
        }
    }
}

fn finish_quality(mut quality: QualityMetrics) -> QualityMetrics {
    let passed = quality.verifier_passed + quality.review_passed;
    let total = quality.verifier_total + quality.review_total;
    quality.gate_pass_rate = average(passed, total);
    quality
}

fn trust(project_root: &Path) -> Result<TrustMetrics> {
    let plugins = plugin_registry::list_installed(project_root)?;
    Ok(TrustMetrics {
        installed_plugins: plugins.len(),
        signed_plugins: plugins
            .iter()
            .filter(|plugin| plugin.signature.is_some())
            .count(),
        verified_signatures: plugins
            .iter()
            .filter(|plugin| plugin.signature_verified)
            .count(),
        trusted_plugins: plugins
            .iter()
            .filter(|plugin| plugin.trust == "trusted")
            .count(),
    })
}

fn average(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn average_float(numerator: f64, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator / denominator as f64
    }
}
