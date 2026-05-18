use std::{collections::BTreeSet, path::Path};

use anyhow::Result;

use super::{
    audit::build_report,
    gaps::{readiness_gaps, render_gaps},
    types::{
        AuditOptions, AuditRenderResult, ReadinessAuditReport, ReadinessCheck,
        ReadinessChecklistReport, ReadinessRequirement, ReadinessRequirementCheck,
        ReadinessSources,
    },
};

struct RequirementSpec {
    id: &'static str,
    requirement: &'static str,
    checks: &'static [&'static str],
    artifacts: &'static [&'static str],
}

const REQUIREMENTS: &[RequirementSpec] = &[
    RequirementSpec {
        id: "roadmap_files",
        requirement: "Named API-native and post-1.0 roadmap files are present and wired into the readiness source set.",
        checks: &["api_native_plan", "post_1_0_plan", "repo_roadmap"],
        artifacts: &["source:api_native_plan", "source:post_1_0_plan", "source:repo_roadmap"],
    },
    RequirementSpec {
        id: "api_native_provider_surface",
        requirement: "DeepSeek and Kimi are exposed as the user-facing API-native providers without legacy CLI providers on the main surface.",
        checks: &["provider_surface"],
        artifacts: &["command:agenthub providers status --json", "command:agenthub providers recovery --json"],
    },
    RequirementSpec {
        id: "deepseek_api",
        requirement: "DeepSeek has passed source-backed provider dogfood evidence through AgentHub-owned API calls.",
        checks: &["provider_deepseek"],
        artifacts: &["command:agenthub providers test deepseek", "evidence:dogfood_history"],
    },
    RequirementSpec {
        id: "kimi_api",
        requirement: "Kimi/Moonshot has a rehearsed replacement-key path, a plain OpenAI-compatible API key, passed auth, and passed provider dogfood evidence.",
        checks: &[
            "rc_check_kimi_unblock_rehearsal",
            "provider_kimi",
            "kimi_auth",
        ],
        artifacts: &[
            "command:agenthub providers rehearse-unblock kimi --from-file <new-key-file>",
            "command:scripts/test-kimi-unblock-rehearsal.sh",
            "command:agenthub providers inspect-key kimi",
            "command:agenthub providers preflight-key kimi --from-file <new-key-file>",
            "command:agenthub providers rc-unblock kimi --from-file <new-key-file>",
            "evidence:kimi_auth_report",
            "evidence:dogfood_history",
        ],
    },
    RequirementSpec {
        id: "chat_ops_project_modes",
        requirement: "Chat, Ops, and Project flows have source-backed no-bootstrap, Ops receipt, and project-edit dogfood evidence.",
        checks: &[
            "real_sessions",
            "ops_flows",
            "project_edit_flows",
            "rc_check_chat_no_bootstrap",
            "rc_check_ops_no_bootstrap",
            "rc_check_ops_receipts",
            "rc_check_shell_ux_aliases",
        ],
        artifacts: &[
            "command:scripts/test-shell-ux-aliases.sh",
            "command:AGENTHUB_DOGFOOD_ACCEPTANCE=1 scripts/dogfood.sh",
            "command:agenthub ops exec <safe-command> --jsonl",
            "evidence:rc-evidence.jsonl",
        ],
    },
    RequirementSpec {
        id: "memory_observability",
        requirement: "Memory, resume/rewind, stats, approval UX, cost receipts, and long-session observability are covered by source-backed RC checks.",
        checks: &[
            "cost_receipts",
            "rc_check_resume",
            "rc_check_rewind",
            "rc_check_stats",
            "rc_check_cost_receipts",
            "rc_check_approval_ux",
            "rc_check_long_session_latency",
        ],
        artifacts: &[
            "command:/memory inbox",
            "command:/context",
            "command:agenthub readiness audit --json --check",
            "evidence:rc-evidence.jsonl",
        ],
    },
    RequirementSpec {
        id: "rc_dogfood_gate",
        requirement: "The 1.0 RC dogfood gate has enough real sessions, Ops flows, project-edit flows, cost receipts, provider evidence, and no open blockers.",
        checks: &[
            "real_sessions",
            "ops_flows",
            "project_edit_flows",
            "cost_receipts",
            "provider_deepseek",
            "provider_kimi",
            "open_blockers",
            "rc_dogfood_gate",
        ],
        artifacts: &[
            "command:scripts/rc-evidence-collect.sh",
            "command:scripts/rc-dogfood-gate.sh --check",
            "evidence:rc-evidence.jsonl",
            "evidence:dogfood_history",
        ],
    },
    RequirementSpec {
        id: "post_1_0_sequence",
        requirement: "Post-1.0 MCP/A2A and ecosystem tracks are visible as a planning surface but remain gated until the 1.0 readiness gate passes.",
        checks: &["ecosystem_surfaces"],
        artifacts: &[
            "command:agenthub ecosystem status --json",
            "source:post_1_0_plan",
            "source:repo_roadmap",
        ],
    },
];

pub fn render_checklist(project_root: &Path, options: AuditOptions) -> Result<AuditRenderResult> {
    let audit = build_report(project_root, options.no_refresh)?;
    let checklist = checklist_report(audit);
    let failed = checklist.failed;
    let output = if options.json {
        format!("{}\n", serde_json::to_string_pretty(&checklist)?)
    } else {
        render_checklist_text(&checklist)
    };
    Ok(AuditRenderResult { output, failed })
}

fn checklist_report(audit: ReadinessAuditReport) -> ReadinessChecklistReport {
    let requirements = REQUIREMENTS
        .iter()
        .map(|spec| requirement_from_spec(spec, &audit))
        .collect::<Vec<_>>();
    let failed = requirements
        .iter()
        .any(|requirement| requirement.status != "passed");
    let status = if failed { "incomplete" } else { "ready" }.to_string();
    ReadinessChecklistReport {
        objective: audit.objective,
        status,
        failed,
        blocker_scope: audit.blocker_scope,
        blocker_kinds: audit.blocker_kinds,
        blocked_checks: audit.blocked_checks,
        sources: ReadinessSources {
            api_native_plan: audit.sources.api_native_plan,
            post_1_0_plan: audit.sources.post_1_0_plan,
            repo_roadmap: audit.sources.repo_roadmap,
        },
        evidence: audit.evidence,
        dogfood_history: audit.dogfood_history,
        kimi_auth_report: audit.kimi_auth_report,
        kimi_rc_operator_receipt: audit.kimi_rc_operator_receipt,
        latest_kimi_rc_attempt: audit.latest_kimi_rc_attempt,
        gaps: readiness_gaps(&audit.checks),
        requirements,
        next: if failed { audit.next } else { Vec::new() },
    }
}

fn requirement_from_spec(
    spec: &RequirementSpec,
    audit: &ReadinessAuditReport,
) -> ReadinessRequirement {
    let checks = spec
        .checks
        .iter()
        .filter_map(|id| audit.checks.iter().find(|check| check.id == *id))
        .map(requirement_check)
        .collect::<Vec<_>>();
    let next_commands = spec
        .checks
        .iter()
        .filter_map(|id| audit.checks.iter().find(|check| check.id == *id))
        .filter(|check| check.status != "passed")
        .flat_map(|check| check.next_commands.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let artifacts = spec
        .artifacts
        .iter()
        .map(|artifact| resolve_artifact(artifact, audit))
        .collect::<Vec<_>>();
    let status = requirement_status(&checks);
    let detail = requirement_detail(&checks);

    ReadinessRequirement {
        id: spec.id.to_string(),
        requirement: spec.requirement.to_string(),
        status,
        detail,
        artifacts,
        checks,
        next_commands,
    }
}

fn requirement_check(check: &ReadinessCheck) -> ReadinessRequirementCheck {
    ReadinessRequirementCheck {
        id: check.id.clone(),
        status: check.status.clone(),
        detail: check.detail.clone(),
        blocker_kind: check.blocker_kind.clone(),
    }
}

fn requirement_status(checks: &[ReadinessRequirementCheck]) -> String {
    if checks.is_empty() {
        return "uncovered".to_string();
    }
    if checks.iter().all(|check| check.status == "passed") {
        return "passed".to_string();
    }
    if checks.iter().any(|check| check.status == "blocked") {
        return "blocked".to_string();
    }
    "missing".to_string()
}

fn requirement_detail(checks: &[ReadinessRequirementCheck]) -> String {
    if checks.is_empty() {
        return "no readiness checks mapped".to_string();
    }
    let passed = checks
        .iter()
        .filter(|check| check.status == "passed")
        .count();
    let blocked = checks
        .iter()
        .filter(|check| check.status == "blocked")
        .count();
    let missing = checks
        .iter()
        .filter(|check| check.status == "missing")
        .count();
    format!(
        "checks passed:{passed}/{} blocked:{blocked} missing:{missing}",
        checks.len()
    )
}

fn resolve_artifact(artifact: &str, audit: &ReadinessAuditReport) -> String {
    match artifact {
        "source:api_native_plan" => format!("file:{}", audit.sources.api_native_plan),
        "source:post_1_0_plan" => format!("file:{}", audit.sources.post_1_0_plan),
        "source:repo_roadmap" => format!("file:{}", audit.sources.repo_roadmap),
        "evidence:rc-evidence.jsonl" => format!("file:{}", audit.evidence),
        "evidence:dogfood_history" => format!("file:{}", audit.dogfood_history),
        "evidence:kimi_auth_report" => format!("file:{}", audit.kimi_auth_report),
        other => other.to_string(),
    }
}

fn render_checklist_text(report: &ReadinessChecklistReport) -> String {
    let mut out = String::new();
    out.push_str("AgentHub API-native readiness checklist\n");
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
    render_gaps(&mut out, &report.gaps);
    for requirement in &report.requirements {
        out.push_str(&format!(
            "requirement\t{}\t{}\t{}\n",
            requirement.id, requirement.status, requirement.requirement
        ));
        out.push_str(&format!(
            "requirement_detail\t{}\t{}\n",
            requirement.id, requirement.detail
        ));
        for artifact in &requirement.artifacts {
            out.push_str(&format!(
                "requirement_artifact\t{}\t{}\n",
                requirement.id, artifact
            ));
        }
        for check in &requirement.checks {
            out.push_str(&format!(
                "requirement_check\t{}\t{}\t{}\t{}\n",
                requirement.id, check.id, check.status, check.detail
            ));
            if let Some(kind) = &check.blocker_kind {
                out.push_str(&format!(
                    "requirement_check_blocker_kind\t{}\t{}\t{}\n",
                    requirement.id, check.id, kind
                ));
            }
        }
        for (index, command) in requirement.next_commands.iter().enumerate() {
            out.push_str(&format!(
                "requirement_next\t{}\t{}\t{}\n",
                requirement.id,
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
