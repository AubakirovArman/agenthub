use std::{collections::BTreeSet, path::Path};

use anyhow::Result;
use serde::Serialize;

use crate::product_cli::{ecosystem, version};

use super::{
    audit::build_report,
    operator_receipt,
    types::{AuditOptions, AuditRenderResult, ReadinessAuditReport, ReadinessSources, OBJECTIVE},
};

#[derive(Debug, Serialize)]
struct ReadinessNextReport {
    objective: String,
    package_version: String,
    status: String,
    failed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_scope: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocker_kinds: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blocked_checks: Vec<String>,
    phase: String,
    focus: String,
    stop_reason: String,
    next_milestone: String,
    sources: ReadinessSources,
    evidence: String,
    dogfood_history: String,
    kimi_auth_report: String,
    kimi_rc_operator_receipt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_kimi_rc_attempt: Option<super::types::KimiRcOperatorReceiptSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    immediate_commands: Vec<String>,
    verification_commands: Vec<String>,
    deferred_tracks: Vec<DeferredTrack>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    all_next: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DeferredTrack {
    id: String,
    priority: String,
    gate: String,
    depends_on: String,
    next_files: String,
}

pub fn render_next(project_root: &Path, options: AuditOptions) -> Result<AuditRenderResult> {
    let audit = build_report(project_root, options.no_refresh)?;
    let report = next_report(audit);
    let failed = report.failed;
    let output = if options.json {
        format!("{}\n", serde_json::to_string_pretty(&report)?)
    } else {
        render_next_text(&report)
    };
    Ok(AuditRenderResult { output, failed })
}

fn next_report(audit: ReadinessAuditReport) -> ReadinessNextReport {
    let phase = phase(&audit);
    let immediate_commands = prioritized_commands(&audit);
    let verification_commands = verification_commands(&audit);
    let failed = audit.failed;
    let status = if failed { "blocked" } else { "ready" }.to_string();

    ReadinessNextReport {
        objective: OBJECTIVE.to_string(),
        package_version: version().to_string(),
        status,
        failed,
        blocker_scope: audit.blocker_scope,
        blocker_kinds: audit.blocker_kinds,
        blocked_checks: audit.blocked_checks,
        phase: phase.id.to_string(),
        focus: phase.focus.to_string(),
        stop_reason: phase.stop_reason.to_string(),
        next_milestone: phase.next_milestone.to_string(),
        sources: audit.sources,
        evidence: audit.evidence,
        dogfood_history: audit.dogfood_history,
        kimi_auth_report: audit.kimi_auth_report,
        kimi_rc_operator_receipt: audit.kimi_rc_operator_receipt,
        latest_kimi_rc_attempt: audit.latest_kimi_rc_attempt,
        immediate_commands,
        verification_commands,
        deferred_tracks: deferred_tracks(),
        all_next: if failed { audit.next } else { Vec::new() },
    }
}

struct Phase {
    id: &'static str,
    focus: &'static str,
    stop_reason: &'static str,
    next_milestone: &'static str,
}

fn phase(audit: &ReadinessAuditReport) -> Phase {
    let has = |id: &str| audit.blocked_checks.iter().any(|check| check == id);
    if !audit.failed {
        return Phase {
            id: "ready_for_1_0_rc",
            focus: "cut or rehearse the 1.0 RC",
            stop_reason: "all API-native readiness checks are passed",
            next_milestone: "1.0 RC release gate",
        };
    }
    if has("kimi_auth") {
        return Phase {
            id: "external_kimi_credential_unblock",
            focus: "replace the Kimi/Moonshot credential with a plain API key",
            stop_reason: "Kimi auth is blocked by external credential state",
            next_milestone: "passed Kimi auth and provider dogfood evidence",
        };
    }
    if has("provider_kimi") {
        return Phase {
            id: "kimi_provider_dogfood",
            focus: "collect passed Kimi provider dogfood evidence",
            stop_reason: "the Kimi provider has not produced source-backed passed dogfood evidence",
            next_milestone: "provider_kimi readiness check passed",
        };
    }
    if audit
        .blocked_checks
        .iter()
        .any(|id| id == "real_sessions" || id == "ops_flows" || id == "project_edit_flows")
    {
        return Phase {
            id: "collect_rc_usage_evidence",
            focus: "collect real Chat/Ops/Project dogfood sessions",
            stop_reason: "the RC evidence ledger is below one or more usage thresholds",
            next_milestone: "100+ real sessions with required Ops and project-edit coverage",
        };
    }
    if has("open_blockers") {
        return Phase {
            id: "close_open_rc_blockers",
            focus: "close source-backed blocker or critical RC items",
            stop_reason: "the RC evidence ledger still contains open blocker or critical items",
            next_milestone: "zero open blocker/critical RC entries",
        };
    }
    if has("rc_dogfood_gate") {
        return Phase {
            id: "pass_rc_dogfood_gate",
            focus: "rerun evidence collection and the RC dogfood gate",
            stop_reason: "the final 1.0 RC dogfood gate is not passed",
            next_milestone: "scripts/rc-dogfood-gate.sh --check passed",
        };
    }
    Phase {
        id: "clear_readiness_blockers",
        focus: "clear the remaining readiness blockers",
        stop_reason: "one or more readiness checks are incomplete",
        next_milestone: "agenthub readiness audit --json --check passed",
    }
}

fn prioritized_commands(audit: &ReadinessAuditReport) -> Vec<String> {
    let priority = [
        "kimi_auth",
        "provider_kimi",
        "open_blockers",
        "rc_dogfood_gate",
        "real_sessions",
        "ops_flows",
        "project_edit_flows",
        "cost_receipts",
    ];
    let mut commands = Vec::new();
    let mut seen = BTreeSet::new();

    for id in priority {
        if let Some(check) = audit
            .checks
            .iter()
            .find(|check| check.id == id && check.status != "passed")
        {
            push_commands(&mut commands, &mut seen, &check.next_commands);
        }
    }
    for check in audit.checks.iter().filter(|check| check.status != "passed") {
        push_commands(&mut commands, &mut seen, &check.next_commands);
    }
    commands.truncate(8);
    commands
}

fn push_commands(commands: &mut Vec<String>, seen: &mut BTreeSet<String>, items: &[String]) {
    for command in items {
        if seen.insert(command.clone()) {
            commands.push(command.clone());
        }
    }
}

fn verification_commands(audit: &ReadinessAuditReport) -> Vec<String> {
    if !audit.failed {
        return vec![
            "agenthub readiness evidence --json --check".to_string(),
            "agenthub readiness audit --json --check".to_string(),
        ];
    }
    vec![
        "agenthub readiness blockers --json --check".to_string(),
        "agenthub readiness checklist --json --check".to_string(),
        "agenthub readiness evidence --json --check".to_string(),
        "agenthub readiness audit --json --check".to_string(),
    ]
}

fn deferred_tracks() -> Vec<DeferredTrack> {
    ecosystem::surfaces()
        .into_iter()
        .map(|surface| DeferredTrack {
            id: surface.id.to_string(),
            priority: surface.priority.to_string(),
            gate: surface.gate.to_string(),
            depends_on: surface.depends_on.to_string(),
            next_files: surface.next_files.to_string(),
        })
        .collect()
}

fn render_next_text(report: &ReadinessNextReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub readiness next\n");
    out.push_str(&format!("objective\t{}\n", report.objective));
    out.push_str(&format!("package_version\t{}\n", report.package_version));
    out.push_str(&format!("phase\t{}\n", report.phase));
    out.push_str(&format!("focus\t{}\n", report.focus));
    out.push_str(&format!("stop_reason\t{}\n", report.stop_reason));
    out.push_str(&format!("next_milestone\t{}\n", report.next_milestone));
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
    out.push_str(&format!("dogfood_history\t{}\n", report.dogfood_history));
    out.push_str(&format!("kimi_auth_report\t{}\n", report.kimi_auth_report));
    out.push_str(&format!(
        "kimi_rc_operator_receipt\t{}\n",
        report.kimi_rc_operator_receipt
    ));
    if let Some(summary) = &report.latest_kimi_rc_attempt {
        operator_receipt::render_summary(&mut out, summary);
    }
    for (index, command) in report.immediate_commands.iter().enumerate() {
        out.push_str(&format!("immediate\t{}\t{}\n", index + 1, command));
    }
    for (index, command) in report.verification_commands.iter().enumerate() {
        out.push_str(&format!("verify\t{}\t{}\n", index + 1, command));
    }
    for track in &report.deferred_tracks {
        out.push_str(&format!(
            "deferred_track\t{}\t{}\t{}\t{}\n",
            track.id, track.priority, track.gate, track.next_files
        ));
    }
    out.push_str(&format!("status\t{}\n", report.status));
    for (index, command) in report.all_next.iter().enumerate() {
        out.push_str(&format!("next\t{}\t{}\n", index + 1, command));
    }
    out
}
