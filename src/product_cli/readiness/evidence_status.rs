use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use super::{
    audit::build_report,
    gaps::{readiness_gaps, render_gaps},
    types::{
        env_usize, AuditOptions, AuditRenderResult, ReadinessAuditReport, ReadinessCheck,
        ReadinessGap, ReadinessMetrics, OBJECTIVE,
    },
};

#[derive(Debug, Serialize)]
struct EvidenceStatusReport {
    objective: String,
    status: String,
    failed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_scope: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocker_kinds: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocked_checks: Vec<String>,
    evidence: String,
    dogfood_history: String,
    kimi_auth_report: String,
    kimi_rc_operator_receipt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_kimi_rc_attempt: Option<super::types::KimiRcOperatorReceiptSummary>,
    metrics: ReadinessMetrics,
    history: DogfoodHistoryStatus,
    thresholds: Vec<EvidenceThreshold>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    gaps: Vec<ReadinessGap>,
    providers: Vec<EvidenceItem>,
    kimi_auth: EvidenceItem,
    rc_checks: Vec<EvidenceItem>,
    open_blockers: EvidenceItem,
    gate: EvidenceItem,
    next: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DogfoodHistoryStatus {
    status: String,
    suite_runs: usize,
    required_suite_runs: usize,
    provider_passed_runs: usize,
    required_provider_passed_runs: usize,
    distinct_days: usize,
    required_distinct_days: usize,
    missing_reports: usize,
}

#[derive(Debug, Serialize)]
struct EvidenceThreshold {
    id: String,
    status: String,
    actual: usize,
    required: usize,
    missing: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    next_commands: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EvidenceItem {
    id: String,
    status: String,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_kind: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    next_commands: Vec<String>,
}

pub fn render_evidence(project_root: &Path, options: AuditOptions) -> Result<AuditRenderResult> {
    let audit = build_report(project_root, options.no_refresh)?;
    let report = evidence_status_report(audit)?;
    let failed = report.failed;
    let output = if options.json {
        format!("{}\n", serde_json::to_string_pretty(&report)?)
    } else {
        render_evidence_text(&report)
    };
    Ok(AuditRenderResult { output, failed })
}

fn evidence_status_report(audit: ReadinessAuditReport) -> Result<EvidenceStatusReport> {
    let checks = audit
        .checks
        .iter()
        .map(|check| (check.id.as_str(), check))
        .collect::<BTreeMap<_, _>>();
    let history = dogfood_history_status(Path::new(&audit.dogfood_history))?;
    let thresholds = threshold_items(&audit.metrics, &checks);
    let providers = prefixed_items("provider_", &checks);
    let kimi_auth = item_from_check(
        "kimi_auth",
        checks.get("kimi_auth").copied(),
        "not evaluated",
    );
    let rc_checks = prefixed_items("rc_check_", &checks);
    let open_blockers = item_from_check(
        "open_blockers",
        checks.get("open_blockers").copied(),
        "not evaluated",
    );
    let gate = item_from_check(
        "rc_dogfood_gate",
        checks.get("rc_dogfood_gate").copied(),
        "not evaluated",
    );
    let failed = audit.failed;
    let status = if failed { "incomplete" } else { "ready" }.to_string();
    let gaps = readiness_gaps(&audit.checks);

    Ok(EvidenceStatusReport {
        objective: OBJECTIVE.to_string(),
        status,
        failed,
        blocker_scope: audit.blocker_scope,
        blocker_kinds: audit.blocker_kinds,
        blocked_checks: audit.blocked_checks,
        evidence: audit.evidence,
        dogfood_history: audit.dogfood_history,
        kimi_auth_report: audit.kimi_auth_report,
        kimi_rc_operator_receipt: audit.kimi_rc_operator_receipt,
        latest_kimi_rc_attempt: audit.latest_kimi_rc_attempt,
        metrics: audit.metrics,
        history,
        thresholds,
        gaps,
        providers,
        kimi_auth,
        rc_checks,
        open_blockers,
        gate,
        next: if failed { audit.next } else { Vec::new() },
    })
}

fn threshold_items(
    metrics: &ReadinessMetrics,
    checks: &BTreeMap<&str, &ReadinessCheck>,
) -> Vec<EvidenceThreshold> {
    [
        (
            "real_sessions",
            metrics.real_sessions,
            metrics.required_sessions,
        ),
        ("ops_flows", metrics.ops_flows, metrics.required_ops_flows),
        (
            "project_edit_flows",
            metrics.project_edit_flows,
            metrics.required_project_edit_flows,
        ),
        (
            "cost_receipts",
            metrics.cost_receipts,
            metrics.required_cost_receipts,
        ),
    ]
    .into_iter()
    .map(|(id, actual, required)| {
        let check = checks.get(id).copied();
        EvidenceThreshold {
            id: id.to_string(),
            status: check
                .map(|check| check.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            actual,
            required,
            missing: required.saturating_sub(actual),
            next_commands: check
                .map(|check| check.next_commands.clone())
                .unwrap_or_default(),
        }
    })
    .collect()
}

fn prefixed_items(prefix: &str, checks: &BTreeMap<&str, &ReadinessCheck>) -> Vec<EvidenceItem> {
    checks
        .iter()
        .filter_map(|(id, check)| {
            id.strip_prefix(prefix).and_then(|short| {
                if short == "surface" {
                    None
                } else {
                    Some(item_from_check(short, Some(check), "not evaluated"))
                }
            })
        })
        .collect()
}

fn item_from_check(id: &str, check: Option<&ReadinessCheck>, missing_detail: &str) -> EvidenceItem {
    if let Some(check) = check {
        return EvidenceItem {
            id: id.to_string(),
            status: check.status.clone(),
            detail: check.detail.clone(),
            blocker_kind: check.blocker_kind.clone(),
            next_commands: check.next_commands.clone(),
        };
    }
    EvidenceItem {
        id: id.to_string(),
        status: "missing".to_string(),
        detail: missing_detail.to_string(),
        blocker_kind: None,
        next_commands: Vec::new(),
    }
}

fn dogfood_history_status(path: &Path) -> Result<DogfoodHistoryStatus> {
    let events = read_jsonl(path)?;
    let required_suite_runs = env_usize(&["AGENTHUB_DOGFOOD_MIN_SUITE_RUNS"], 3);
    let required_provider_passed_runs = env_usize(&["AGENTHUB_DOGFOOD_MIN_PROVIDER_PASSED"], 1);
    let required_distinct_days = env_usize(&["AGENTHUB_DOGFOOD_MIN_DAYS"], 2);
    let suite_runs = events
        .iter()
        .filter(|event| text(event, "kind") == Some("suite"))
        .count();
    let provider_passed_runs = events
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
    let ready = suite_runs >= required_suite_runs
        && provider_passed_runs >= required_provider_passed_runs
        && distinct_days >= required_distinct_days
        && missing_reports == 0;

    Ok(DogfoodHistoryStatus {
        status: if ready { "ready" } else { "incomplete" }.to_string(),
        suite_runs,
        required_suite_runs,
        provider_passed_runs,
        required_provider_passed_runs,
        distinct_days,
        required_distinct_days,
        missing_reports,
    })
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

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn render_evidence_text(report: &EvidenceStatusReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub RC evidence status\n");
    out.push_str(&format!("objective\t{}\n", report.objective));
    if let Some(scope) = &report.blocker_scope {
        out.push_str(&format!("blocker_scope\t{}\n", scope));
    }
    if !report.blocker_kinds.is_empty() {
        out.push_str(&format!(
            "blocker_kinds\t{}\n",
            report.blocker_kinds.join(",")
        ));
    }
    if !report.blocked_checks.is_empty() {
        out.push_str(&format!(
            "blocked_checks\t{}\n",
            report.blocked_checks.join(",")
        ));
    }
    out.push_str(&format!("evidence\t{}\n", report.evidence));
    out.push_str(&format!("dogfood_history\t{}\n", report.dogfood_history));
    out.push_str(&format!("kimi_auth_report\t{}\n", report.kimi_auth_report));
    out.push_str(&format!(
        "history\t{}\tsuite_runs:{}/{} provider_passed:{}/{} days:{}/{} missing_reports:{}\n",
        report.history.status,
        report.history.suite_runs,
        report.history.required_suite_runs,
        report.history.provider_passed_runs,
        report.history.required_provider_passed_runs,
        report.history.distinct_days,
        report.history.required_distinct_days,
        report.history.missing_reports
    ));
    for threshold in &report.thresholds {
        out.push_str(&format!(
            "threshold\t{}\t{}\t{}/{}\tmissing:{}\n",
            threshold.id, threshold.status, threshold.actual, threshold.required, threshold.missing
        ));
        for (index, command) in threshold.next_commands.iter().enumerate() {
            out.push_str(&format!(
                "threshold_next\t{}\t{}\t{}\n",
                threshold.id,
                index + 1,
                command
            ));
        }
    }
    render_gaps(&mut out, &report.gaps);
    for provider in &report.providers {
        render_item(&mut out, "provider", provider);
    }
    render_item(&mut out, "kimi_auth", &report.kimi_auth);
    for check in &report.rc_checks {
        render_item(&mut out, "rc_check", check);
    }
    render_item(&mut out, "open_blockers", &report.open_blockers);
    render_item(&mut out, "gate", &report.gate);
    out.push_str(&format!("status\t{}\n", report.status));
    for (index, command) in report.next.iter().enumerate() {
        out.push_str(&format!("next\t{}\t{}\n", index + 1, command));
    }
    out
}

fn render_item(out: &mut String, label: &str, item: &EvidenceItem) {
    out.push_str(&format!(
        "{}\t{}\t{}\t{}\n",
        label, item.id, item.status, item.detail
    ));
    if let Some(kind) = &item.blocker_kind {
        out.push_str(&format!("{}_blocker_kind\t{}\t{}\n", label, item.id, kind));
    }
    for (index, command) in item.next_commands.iter().enumerate() {
        out.push_str(&format!(
            "{}_next\t{}\t{}\t{}\n",
            label,
            item.id,
            index + 1,
            command
        ));
    }
}
