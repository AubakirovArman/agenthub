use std::path::Path;

use anyhow::Result;

use super::{
    audit::build_report,
    types::{
        AuditOptions, AuditRenderResult, ReadinessBlocker, ReadinessBlockerReport, ReadinessCheck,
        ReadinessSources,
    },
};

pub fn render_blockers(project_root: &Path, options: AuditOptions) -> Result<AuditRenderResult> {
    let report = build_report(project_root, options.no_refresh)?;
    let blockers = report
        .checks
        .iter()
        .filter(|check| check.status != "passed")
        .map(blocker_from_check)
        .collect::<Vec<_>>();
    let failed = !blockers.is_empty();
    let blocker_report = ReadinessBlockerReport {
        objective: report.objective,
        status: if failed { "blocked" } else { "clear" }.to_string(),
        failed,
        blocker_scope: report.blocker_scope,
        blocker_kinds: report.blocker_kinds,
        sources: ReadinessSources {
            api_native_plan: report.sources.api_native_plan,
            post_1_0_plan: report.sources.post_1_0_plan,
            repo_roadmap: report.sources.repo_roadmap,
        },
        evidence: report.evidence,
        dogfood_history: report.dogfood_history,
        kimi_auth_report: report.kimi_auth_report,
        metrics: report.metrics,
        blockers,
        next: if failed { report.next } else { Vec::new() },
    };
    let output = if options.json {
        format!("{}\n", serde_json::to_string_pretty(&blocker_report)?)
    } else {
        render_blockers_text(&blocker_report)
    };
    Ok(AuditRenderResult { output, failed })
}

fn render_blockers_text(report: &ReadinessBlockerReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub readiness blockers\n");
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
    out.push_str(&format!("evidence\t{}\n", report.evidence));
    out.push_str(&format!("dogfood_history\t{}\n", report.dogfood_history));
    out.push_str(&format!("kimi_auth_report\t{}\n", report.kimi_auth_report));
    out.push_str(&format!(
        "metrics\treal_sessions\t{}/{}\n",
        report.metrics.real_sessions, report.metrics.required_sessions
    ));
    out.push_str(&format!(
        "metrics\tops_flows\t{}/{}\n",
        report.metrics.ops_flows, report.metrics.required_ops_flows
    ));
    out.push_str(&format!(
        "metrics\tproject_edit_flows\t{}/{}\n",
        report.metrics.project_edit_flows, report.metrics.required_project_edit_flows
    ));
    out.push_str(&format!(
        "metrics\tcost_receipts\t{}/{}\n",
        report.metrics.cost_receipts, report.metrics.required_cost_receipts
    ));
    out.push_str(&format!(
        "metrics\topen_blockers\t{}\n",
        report.metrics.open_blockers
    ));
    if report.blockers.is_empty() {
        out.push_str("blockers\tclear\n");
    } else {
        for blocker in &report.blockers {
            out.push_str(&format!(
                "blocker\t{}\t{}\t{}\n",
                blocker.id, blocker.status, blocker.detail
            ));
            if let Some(kind) = &blocker.blocker_kind {
                out.push_str(&format!("blocker_kind\t{}\t{}\n", blocker.id, kind));
            }
            for (index, command) in blocker.next_commands.iter().enumerate() {
                out.push_str(&format!(
                    "blocker_next\t{}\t{}\t{}\n",
                    blocker.id,
                    index + 1,
                    command
                ));
            }
        }
    }
    out.push_str(&format!("status\t{}\n", report.status));
    for (index, command) in report.next.iter().enumerate() {
        out.push_str(&format!("next\t{}\t{}\n", index + 1, command));
    }
    out
}

fn blocker_from_check(check: &ReadinessCheck) -> ReadinessBlocker {
    ReadinessBlocker {
        id: check.id.clone(),
        status: check.status.clone(),
        detail: check.detail.clone(),
        blocker_kind: check.blocker_kind.clone(),
        next_commands: check.next_commands.clone(),
    }
}
