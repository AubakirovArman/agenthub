use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use anyhow::Result;
use serde_json::Value;

use crate::product_cli::{ecosystem, providers};

use super::{
    evidence::refresh_evidence,
    next::{check_blocker_kind, check_next_commands},
    render::render_text,
    types::{
        env_usize, next_commands, AuditConfig, AuditOptions, AuditRenderResult, EvidenceSummary,
        ReadinessAuditReport, ReadinessCheck, ReadinessMetrics, ReadinessSources, OBJECTIVE,
    },
};

pub fn render_audit(project_root: &Path, options: AuditOptions) -> Result<AuditRenderResult> {
    let report = build_report(project_root, options.no_refresh)?;
    let failed = report.failed;
    let output = if options.json {
        format!("{}\n", serde_json::to_string_pretty(&report)?)
    } else {
        render_text(&report)
    };
    Ok(AuditRenderResult { output, failed })
}

pub(super) fn build_report(project_root: &Path, no_refresh: bool) -> Result<ReadinessAuditReport> {
    let config = AuditConfig::from_env(project_root);
    if !no_refresh {
        refresh_evidence(project_root, &config.evidence)?;
    }
    audit_report(project_root, &config)
}

fn audit_report(project_root: &Path, config: &AuditConfig) -> Result<ReadinessAuditReport> {
    let evidence = summarize_evidence(&config.evidence)?;
    let history_index = config.history_dir.join("index.jsonl");
    let mut providers_passed = evidence.providers_passed.clone();
    add_history_providers(&history_index, &mut providers_passed)?;

    let metrics = ReadinessMetrics {
        real_sessions: evidence.real_sessions,
        required_sessions: config.min_sessions,
        ops_flows: evidence.ops_flows,
        required_ops_flows: config.min_ops,
        project_edit_flows: evidence.project_edit_flows,
        required_project_edit_flows: config.min_project,
        cost_receipts: evidence.cost_receipts,
        required_cost_receipts: config.min_cost,
        open_blockers: evidence.open_blockers,
    };

    let mut checks = Vec::new();
    push_file_check(&mut checks, "api_native_plan", &config.v04_plan);
    push_file_check(&mut checks, "post_1_0_plan", &config.after_plan);
    push_file_check(&mut checks, "repo_roadmap", &config.roadmap_doc);
    push_check(
        &mut checks,
        "ecosystem_surfaces",
        ecosystem_surface_status(),
        "all post-1.0 roadmap surfaces are exposed by ecosystem status",
        "ecosystem status must expose every post-1.0 roadmap surface",
    );
    push_check(
        &mut checks,
        "provider_surface",
        provider_surface_status(project_root)?,
        "DeepSeek/Kimi are visible without legacy CLI providers in providers status",
        "providers status must show only API-native DeepSeek/Kimi on the main surface",
    );
    push_threshold_check(
        &mut checks,
        "real_sessions",
        metrics.real_sessions,
        metrics.required_sessions,
    );
    push_threshold_check(
        &mut checks,
        "ops_flows",
        metrics.ops_flows,
        metrics.required_ops_flows,
    );
    push_threshold_check(
        &mut checks,
        "project_edit_flows",
        metrics.project_edit_flows,
        metrics.required_project_edit_flows,
    );
    push_threshold_check(
        &mut checks,
        "cost_receipts",
        metrics.cost_receipts,
        metrics.required_cost_receipts,
    );
    for provider in &config.required_providers {
        if providers_passed.contains(provider) {
            push_passed(
                &mut checks,
                &format!("provider_{provider}"),
                "provider dogfood evidence found",
            );
        } else {
            push_blocked(
                &mut checks,
                &format!("provider_{provider}"),
                "missing passed provider dogfood evidence",
            );
        }
    }
    for check_id in &config.required_checks {
        if evidence.checks_passed.contains(check_id) {
            push_passed(
                &mut checks,
                &format!("rc_check_{check_id}"),
                "source-backed check evidence found",
            );
        } else {
            push_missing(
                &mut checks,
                &format!("rc_check_{check_id}"),
                "missing source-backed RC check evidence",
            );
        }
    }
    if metrics.open_blockers == 0 {
        push_passed(&mut checks, "open_blockers", "0 blocker/critical open");
    } else {
        push_blocked(
            &mut checks,
            "open_blockers",
            &open_blockers_detail(&evidence),
        );
    }
    push_kimi_auth_check(&mut checks, &config.kimi_report)?;
    if rc_gate_ready(
        &metrics,
        config,
        &providers_passed,
        &evidence.checks_passed,
        &history_index,
    )? {
        push_passed(
            &mut checks,
            "rc_dogfood_gate",
            "scripts/rc-dogfood-gate.sh --check passed",
        );
    } else {
        push_blocked(
            &mut checks,
            "rc_dogfood_gate",
            "1.0 RC dogfood gate: incomplete",
        );
    }

    let failed = checks.iter().any(|check| check.status != "passed");
    let status = if failed { "incomplete" } else { "ready" }.to_string();
    Ok(ReadinessAuditReport {
        objective: OBJECTIVE.to_string(),
        status,
        failed,
        sources: ReadinessSources {
            api_native_plan: config.v04_plan.display().to_string(),
            post_1_0_plan: config.after_plan.display().to_string(),
            repo_roadmap: config.roadmap_doc.display().to_string(),
        },
        evidence: config.evidence.display().to_string(),
        dogfood_history: history_index.display().to_string(),
        kimi_auth_report: config.kimi_report.display().to_string(),
        metrics,
        checks,
        next: if failed { next_commands() } else { Vec::new() },
    })
}

fn summarize_evidence(path: &Path) -> Result<EvidenceSummary> {
    let mut summary = EvidenceSummary::default();
    for event in read_jsonl(path)? {
        let kind = text(&event, "kind");
        let status = text(&event, "status");
        if kind == Some("session") && status == Some("passed") {
            summary.real_sessions += 1;
            let mode = text(&event, "mode").unwrap_or_default();
            let flow = text(&event, "flow").unwrap_or_default();
            if mode == "ops" || flow == "ops" {
                summary.ops_flows += 1;
            }
            if mode == "project_edit" || flow == "project_edit" {
                summary.project_edit_flows += 1;
            }
            if bool_field(&event, "cost_receipt") {
                summary.cost_receipts += 1;
            }
        }
        if kind == Some("provider") && status == Some("passed") {
            if let Some(provider) = text(&event, "provider") {
                summary.providers_passed.insert(provider.to_string());
            }
        }
        if kind == Some("check") && status == Some("passed") {
            if let Some(id) = text(&event, "id") {
                summary.checks_passed.insert(id.to_string());
            }
        }
        if kind == Some("blocker") && !matches!(status, Some("closed" | "resolved")) {
            let severity = text(&event, "severity");
            if matches!(severity, Some("blocker" | "critical")) {
                summary.open_blockers += 1;
                if let Some(id) = text(&event, "id").filter(|value| !value.is_empty()) {
                    summary.open_blocker_ids.insert(id.to_string());
                }
            }
        }
    }
    Ok(summary)
}

fn open_blockers_detail(evidence: &EvidenceSummary) -> String {
    let count = evidence.open_blockers;
    if evidence.open_blocker_ids.is_empty() {
        return format!("{count} blocker/critical open");
    }
    let ids = evidence
        .open_blocker_ids
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");
    format!("{count} blocker/critical open: {ids}")
}

fn add_history_providers(path: &Path, providers_passed: &mut BTreeSet<String>) -> Result<()> {
    for event in read_jsonl(path)? {
        if text(&event, "kind") == Some("provider")
            && text(&event, "provider_status") == Some("passed")
        {
            if let Some(provider) = text(&event, "provider") {
                providers_passed.insert(provider.to_string());
            }
        }
    }
    Ok(())
}

fn dogfood_history_ready(index: &Path) -> Result<bool> {
    let events = read_jsonl(index)?;
    if events.is_empty() {
        return Ok(false);
    }
    let min_suite = env_usize(&["AGENTHUB_DOGFOOD_MIN_SUITE_RUNS"], 3);
    let min_provider = env_usize(&["AGENTHUB_DOGFOOD_MIN_PROVIDER_PASSED"], 1);
    let min_days = env_usize(&["AGENTHUB_DOGFOOD_MIN_DAYS"], 2);
    let suite_runs = events
        .iter()
        .filter(|event| text(event, "kind") == Some("suite"))
        .count();
    let provider_passed = events
        .iter()
        .filter(|event| {
            text(event, "kind") == Some("provider")
                && text(event, "provider_status") == Some("passed")
        })
        .count();
    let distinct_days = events
        .iter()
        .filter_map(|event| text(event, "archived_at"))
        .filter_map(|value| value.split_once('T').map(|(day, _)| day.to_string()))
        .collect::<HashSet<_>>()
        .len();
    let missing_reports = events
        .iter()
        .filter_map(|event| text(event, "report"))
        .filter(|report| !report.is_empty())
        .filter(|report| !Path::new(report).exists())
        .count();

    Ok(suite_runs >= min_suite
        && provider_passed >= min_provider
        && distinct_days >= min_days
        && missing_reports == 0)
}

fn rc_gate_ready(
    metrics: &ReadinessMetrics,
    config: &AuditConfig,
    providers_passed: &BTreeSet<String>,
    checks_passed: &BTreeSet<String>,
    history_index: &Path,
) -> Result<bool> {
    Ok(dogfood_history_ready(history_index)?
        && metrics.real_sessions >= metrics.required_sessions
        && metrics.ops_flows >= metrics.required_ops_flows
        && metrics.project_edit_flows >= metrics.required_project_edit_flows
        && metrics.cost_receipts >= metrics.required_cost_receipts
        && metrics.open_blockers == 0
        && config
            .required_providers
            .iter()
            .all(|provider| providers_passed.contains(provider))
        && config
            .required_checks
            .iter()
            .all(|check| checks_passed.contains(check)))
}

fn provider_surface_status(project_root: &Path) -> Result<bool> {
    if let Ok(status) = std::env::var("AGENTHUB_API_AUDIT_PROVIDER_STATUS") {
        if !status.trim().is_empty() {
            let has_deepseek = line_starts_with(&status, "deepseek");
            let has_kimi = line_starts_with(&status, "kimi");
            let has_legacy = ["codex", "gemini", "kimi-api", "command"]
                .iter()
                .any(|id| line_starts_with(&status, id));
            return Ok(has_deepseek && has_kimi && !has_legacy);
        }
    }
    let ids = providers::statuses(project_root)?
        .into_iter()
        .map(|status| status.info.id)
        .collect::<Vec<_>>();
    Ok(ids.iter().any(|id| id == "deepseek")
        && ids.iter().any(|id| id == "kimi")
        && !ids
            .iter()
            .any(|id| matches!(id.as_str(), "codex" | "gemini" | "kimi-api" | "command")))
}

fn ecosystem_surface_status() -> bool {
    let required = [
        "mcp",
        "a2a",
        "subagents-v2",
        "async-background-agents",
        "ollama-local-llm",
        "multimodal-context",
        "team-collaboration",
        "enterprise-marketplace",
    ];
    let ids = ecosystem::surfaces()
        .into_iter()
        .map(|surface| surface.id)
        .collect::<HashSet<_>>();
    required.iter().all(|id| ids.contains(id))
}

fn line_starts_with(text: &str, id: &str) -> bool {
    text.lines().any(|line| {
        let mut parts = line.split_whitespace();
        parts.next() == Some(id)
    })
}

fn push_file_check(checks: &mut Vec<ReadinessCheck>, id: &str, path: &Path) {
    if path.is_file() {
        push_passed(checks, id, &path.display().to_string());
    } else {
        push_missing(checks, id, &path.display().to_string());
    }
}

fn push_threshold_check(
    checks: &mut Vec<ReadinessCheck>,
    id: &str,
    actual: usize,
    required: usize,
) {
    let detail = format!("{actual}/{required}");
    if actual >= required {
        push_passed(checks, id, &detail);
    } else {
        push_missing(checks, id, &detail);
    }
}

fn push_kimi_auth_check(checks: &mut Vec<ReadinessCheck>, path: &Path) -> Result<()> {
    let Some(report) = read_json_file(path)? else {
        push_missing(checks, "kimi_auth", "no Kimi auth report");
        return Ok(());
    };
    match text(&report, "status") {
        Some("passed") => push_passed(checks, "kimi_auth", "Kimi auth passed"),
        Some("blocked") => {
            let mut detail = format!(
                "key:{}",
                text(&report, "auth_key_sha256_12").unwrap_or("unknown")
            );
            if let Some(source) = text(&report, "auth_key_source").filter(|value| !value.is_empty())
            {
                detail.push_str(&format!("; source:{source}"));
            }
            if let Some(warning) =
                text(&report, "credential_warning").filter(|value| !value.is_empty())
            {
                detail.push_str(&format!("; warning:{warning}"));
            }
            let next = text(&report, "next_action")
                .filter(|value| !value.is_empty())
                .unwrap_or("replace or rotate the Kimi/Moonshot API key");
            detail.push_str(&format!("; {next}"));
            push_blocked(checks, "kimi_auth", &detail);
        }
        Some(other) => push_blocked(checks, "kimi_auth", &format!("status:{other}")),
        None => push_missing(checks, "kimi_auth", "no Kimi auth report"),
    }
    Ok(())
}

fn push_check(
    checks: &mut Vec<ReadinessCheck>,
    id: &str,
    passed: bool,
    passed_detail: &str,
    failed_detail: &str,
) {
    if passed {
        push_passed(checks, id, passed_detail);
    } else {
        push_blocked(checks, id, failed_detail);
    }
}

fn push_passed(checks: &mut Vec<ReadinessCheck>, id: &str, detail: &str) {
    checks.push(ReadinessCheck {
        id: id.to_string(),
        status: "passed".to_string(),
        detail: detail.to_string(),
        blocker_kind: None,
        next_commands: Vec::new(),
    });
}

fn push_missing(checks: &mut Vec<ReadinessCheck>, id: &str, detail: &str) {
    checks.push(ReadinessCheck {
        id: id.to_string(),
        status: "missing".to_string(),
        detail: detail.to_string(),
        blocker_kind: check_blocker_kind(id, detail).map(str::to_string),
        next_commands: check_next_commands(id, detail),
    });
}

fn push_blocked(checks: &mut Vec<ReadinessCheck>, id: &str, detail: &str) {
    checks.push(ReadinessCheck {
        id: id.to_string(),
        status: "blocked".to_string(),
        detail: detail.to_string(),
        blocker_kind: check_blocker_kind(id, detail).map(str::to_string),
        next_commands: check_next_commands(id, detail),
    });
}

fn read_jsonl(path: &Path) -> Result<Vec<Value>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(Vec::new());
    };
    let mut values = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            values.push(value);
        }
    }
    Ok(values)
}

fn read_json_file(path: &Path) -> Result<Option<Value>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&text)?))
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}
