use super::types::ReadinessAuditReport;

pub(super) fn render_text(report: &ReadinessAuditReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub API-native readiness audit\n");
    out.push_str(&format!("objective\t{}\n", report.objective));
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
    for check in &report.checks {
        out.push_str(&format!(
            "check\t{}\t{}\t{}\n",
            check.id, check.status, check.detail
        ));
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
