use super::{operator_receipt, types::ReadinessAuditReport};

pub(super) fn render_text(report: &ReadinessAuditReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub API-native readiness audit\n");
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
    for check in &report.checks {
        out.push_str(&format!(
            "check\t{}\t{}\t{}\n",
            check.id, check.status, check.detail
        ));
        if let Some(kind) = &check.blocker_kind {
            out.push_str(&format!("check_blocker_kind\t{}\t{}\n", check.id, kind));
        }
        for (index, command) in check.next_commands.iter().enumerate() {
            out.push_str(&format!(
                "check_next\t{}\t{}\t{}\n",
                check.id,
                index + 1,
                command
            ));
        }
    }
    out.push_str(&format!("status\t{}\n", report.status));
    for (index, command) in report.next.iter().enumerate() {
        out.push_str(&format!("next\t{}\t{}\n", index + 1, command));
    }
    out
}
