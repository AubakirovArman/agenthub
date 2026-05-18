use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::product_cli::{providers, version};

use super::{
    action_plan,
    audit::build_report,
    checklist, evidence_status,
    gaps::{readiness_gaps, render_gaps},
    operator_receipt,
    types::{
        AuditOptions, AuditRenderResult, ReadinessAuditReport, ReadinessGap, ReadinessSources,
        OBJECTIVE,
    },
};

#[derive(Debug, Serialize)]
struct ReadinessCompletionReport {
    objective: String,
    package_version: String,
    status: String,
    failed: bool,
    completion_status: String,
    decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_scope: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocker_kinds: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocked_checks: Vec<String>,
    sources: ReadinessSources,
    evidence: String,
    dogfood_history: String,
    kimi_auth_report: String,
    kimi_rc_operator_receipt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_kimi_rc_attempt: Option<super::types::KimiRcOperatorReceiptSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    gaps: Vec<ReadinessGap>,
    current_action: Value,
    prompt_to_artifact: Value,
    evidence_status: Value,
    provider_statuses: Value,
    verification_commands: Vec<String>,
}

pub fn render_completion(project_root: &Path, options: AuditOptions) -> Result<AuditRenderResult> {
    let audit = build_report(project_root, options.no_refresh)?;
    let report = completion_report(project_root, audit)?;
    let failed = report.failed;
    let output = if options.json {
        format!("{}\n", serde_json::to_string_pretty(&report)?)
    } else {
        render_completion_text(&report)
    };
    Ok(AuditRenderResult { output, failed })
}

fn completion_report(
    project_root: &Path,
    audit: ReadinessAuditReport,
) -> Result<ReadinessCompletionReport> {
    let current_action = rendered_json(action_plan::render_next(
        project_root,
        AuditOptions {
            json: true,
            no_refresh: true,
        },
    )?)?;
    let prompt_to_artifact = rendered_json(checklist::render_checklist(
        project_root,
        AuditOptions {
            json: true,
            no_refresh: true,
        },
    )?)?;
    let evidence_status = rendered_json(evidence_status::render_evidence(
        project_root,
        AuditOptions {
            json: true,
            no_refresh: true,
        },
    )?)?;
    let provider_statuses = serde_json::from_str(&providers::render_status_json(project_root)?)?;
    let verification_commands = current_action
        .get("verification_commands")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let completion_status = completion_status(&audit);
    let decision = completion_decision(&completion_status);
    let gaps = readiness_gaps(&audit.checks);

    Ok(ReadinessCompletionReport {
        objective: OBJECTIVE.to_string(),
        package_version: version().to_string(),
        status: audit.status,
        failed: audit.failed,
        completion_status,
        decision,
        blocker_scope: audit.blocker_scope,
        blocker_kinds: audit.blocker_kinds,
        blocked_checks: audit.blocked_checks,
        sources: audit.sources,
        evidence: audit.evidence,
        dogfood_history: audit.dogfood_history,
        kimi_auth_report: audit.kimi_auth_report,
        kimi_rc_operator_receipt: audit.kimi_rc_operator_receipt,
        latest_kimi_rc_attempt: audit.latest_kimi_rc_attempt,
        gaps,
        current_action,
        prompt_to_artifact,
        evidence_status,
        provider_statuses,
        verification_commands,
    })
}

fn rendered_json(rendered: AuditRenderResult) -> Result<Value> {
    Ok(serde_json::from_str(&rendered.output)?)
}

fn completion_status(audit: &ReadinessAuditReport) -> String {
    if !audit.failed {
        return "complete".to_string();
    }
    match audit.blocker_scope.as_deref() {
        Some("external_only") => "blocked_external".to_string(),
        Some("mixed") => "blocked_mixed".to_string(),
        _ => "blocked_local_or_unknown".to_string(),
    }
}

fn completion_decision(status: &str) -> String {
    match status {
        "complete" => "API-native 1.0 completion requirements are source-backed and ready".to_string(),
        "blocked_external" => {
            "Implementation evidence is present, but completion is blocked by external provider or credential evidence".to_string()
        }
        "blocked_mixed" => {
            "Completion is blocked by a mix of external and local or unknown readiness gaps".to_string()
        }
        _ => "Completion is blocked by local or unknown readiness gaps".to_string(),
    }
}

fn render_completion_text(report: &ReadinessCompletionReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub readiness completion\n");
    out.push_str(&format!("objective\t{}\n", report.objective));
    out.push_str(&format!("package_version\t{}\n", report.package_version));
    out.push_str(&format!(
        "completion_status\t{}\n",
        report.completion_status
    ));
    out.push_str(&format!("decision\t{}\n", report.decision));
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
    out.push_str(&format!(
        "source\tapi_native_plan\t{}\n",
        report.sources.api_native_plan
    ));
    out.push_str(&format!(
        "source\tpost_1_0_plan\t{}\n",
        report.sources.post_1_0_plan
    ));
    out.push_str(&format!(
        "source\trepo_roadmap\t{}\n",
        report.sources.repo_roadmap
    ));
    out.push_str(&format!("evidence\t{}\n", report.evidence));
    out.push_str(&format!(
        "kimi_rc_operator_receipt\t{}\n",
        report.kimi_rc_operator_receipt
    ));
    if let Some(summary) = &report.latest_kimi_rc_attempt {
        operator_receipt::render_summary(&mut out, summary);
    }
    render_gaps(&mut out, &report.gaps);
    render_current_action(&mut out, &report.current_action);
    render_requirements(&mut out, &report.prompt_to_artifact);
    render_provider_statuses(&mut out, &report.provider_statuses);
    render_evidence_status(&mut out, &report.evidence_status);
    for (index, command) in report.verification_commands.iter().enumerate() {
        out.push_str(&format!("verify\t{}\t{}\n", index + 1, command));
    }
    out.push_str(&format!("status\t{}\n", report.status));
    out
}

fn render_current_action(out: &mut String, value: &Value) {
    for key in ["phase", "focus", "stop_reason", "next_milestone"] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            out.push_str(&format!("current_action\t{}\t{}\n", key, text));
        }
    }
    if let Some(commands) = value.get("immediate_commands").and_then(Value::as_array) {
        for (index, command) in commands.iter().filter_map(Value::as_str).enumerate() {
            out.push_str(&format!("immediate\t{}\t{}\n", index + 1, command));
        }
    }
}

fn render_requirements(out: &mut String, value: &Value) {
    if let Some(requirements) = value.get("requirements").and_then(Value::as_array) {
        for requirement in requirements {
            let id = requirement
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let status = requirement
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let detail = requirement
                .get("detail")
                .and_then(Value::as_str)
                .unwrap_or("");
            out.push_str(&format!("requirement\t{}\t{}\t{}\n", id, status, detail));
        }
    }
}

fn render_provider_statuses(out: &mut String, value: &Value) {
    if let Some(providers) = value.as_array() {
        for provider in providers {
            let id = provider
                .get("provider")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let state = provider
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let blocker = provider
                .get("blocker_kind")
                .and_then(Value::as_str)
                .unwrap_or("-");
            out.push_str(&format!("provider\t{}\t{}\t{}\n", id, state, blocker));
        }
    }
}

fn render_evidence_status(out: &mut String, value: &Value) {
    if let Some(history) = value.get("history") {
        let status = history
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let suite_runs = history
            .get("suite_runs")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let days = history
            .get("distinct_days")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        out.push_str(&format!(
            "history\t{}\tsuite_runs:{} days:{}\n",
            status, suite_runs, days
        ));
    }
    if let Some(thresholds) = value.get("thresholds").and_then(Value::as_array) {
        for threshold in thresholds {
            let id = threshold
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let status = threshold
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let actual = threshold.get("actual").and_then(Value::as_u64).unwrap_or(0);
            let required = threshold
                .get("required")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            out.push_str(&format!(
                "threshold\t{}\t{}\t{}/{}\n",
                id, status, actual, required
            ));
        }
    }
}
