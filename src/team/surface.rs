use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::agent_dir;
use crate::analytics::{self, AnalyticsSummary};
use crate::enterprise::{self, ApprovalSummary};
use crate::memory;
use crate::team::{
    ProjectAuditExport, TeamAuditExport, TeamExportWrite, TeamMemorySummary, TeamPayload,
    TeamPolicySummary, TeamProject, TeamReportLink, TeamRunnerSummary, TeamTotals,
};

pub fn collect(projects: &[PathBuf]) -> Result<TeamPayload> {
    let mut out = Vec::new();
    for project in projects {
        out.push(project_summary(project)?);
    }
    Ok(TeamPayload {
        version: "team.surface.v1".to_string(),
        generated_at: Utc::now(),
        totals: totals(&out),
        projects: out,
    })
}

pub fn write_export(projects: &[PathBuf], output_dir: &Path) -> Result<TeamExportWrite> {
    let payload = collect(projects)?;
    let audit = audit_export(projects)?;
    fs::create_dir_all(output_dir).with_context(|| format!("create {}", output_dir.display()))?;
    let payload_path = output_dir.join("team_payload.json");
    let audit_path = output_dir.join("audit_export.json");
    fs::write(&payload_path, serde_json::to_string_pretty(&payload)?)
        .with_context(|| format!("write {}", payload_path.display()))?;
    fs::write(&audit_path, serde_json::to_string_pretty(&audit)?)
        .with_context(|| format!("write {}", audit_path.display()))?;
    Ok(TeamExportWrite {
        payload_path,
        audit_path,
    })
}

pub fn audit_export(projects: &[PathBuf]) -> Result<TeamAuditExport> {
    let mut out = Vec::new();
    for project in projects.iter().filter(|path| path.exists()) {
        out.push(ProjectAuditExport {
            project: project.display().to_string(),
            audit_events: enterprise::list_audit(project, 1000)?,
            reports: reports(project)?,
        });
    }
    Ok(TeamAuditExport {
        version: "team.audit.v1".to_string(),
        generated_at: Utc::now(),
        projects: out,
    })
}

fn project_summary(project: &Path) -> Result<TeamProject> {
    if !project.exists() {
        return Ok(missing_project(project));
    }
    let transactions = agent_dir::list_transactions(project)?;
    let approvals = enterprise::approval_summary(project)?;
    let (policy, source) = enterprise::load_policy_with_source(project)?;
    let runners = enterprise::runner_inventory(project)?;
    let memory = memory::inspect(project)?;
    let analytics = analytics::load_summary(project)?;
    let audit_events = enterprise::list_audit(project, 1000)?.len();
    Ok(TeamProject {
        path: project.display().to_string(),
        exists: true,
        transactions: transactions.len(),
        approvals,
        policy: TeamPolicySummary {
            source_mode: source.mode,
            source_path: source.path,
            enabled: policy.enterprise.enabled,
            default_role: policy.enterprise.default_role,
        },
        runners: TeamRunnerSummary {
            default: runners.default,
            remote_count: runners.remote.len(),
        },
        memory: TeamMemorySummary {
            committed: memory.committed,
            failed_attempts: memory.failed_attempts,
        },
        analytics,
        audit_events,
        reports: reports(project)?,
    })
}

fn missing_project(project: &Path) -> TeamProject {
    TeamProject {
        path: project.display().to_string(),
        exists: false,
        transactions: 0,
        approvals: ApprovalSummary::default(),
        policy: TeamPolicySummary::default(),
        runners: TeamRunnerSummary::default(),
        memory: TeamMemorySummary::default(),
        analytics: AnalyticsSummary::default(),
        audit_events: 0,
        reports: Vec::new(),
    }
}

fn reports(project: &Path) -> Result<Vec<TeamReportLink>> {
    let mut out = Vec::new();
    for row in agent_dir::list_transactions(project)? {
        out.push(TeamReportLink {
            kind: "transaction".to_string(),
            path: row.report_path.display().to_string(),
        });
    }
    let enterprise_dir = project.join(agent_dir::AGENT_DIR).join("enterprise");
    if enterprise_dir.exists() {
        for entry in fs::read_dir(&enterprise_dir)? {
            let path = entry?.path();
            if is_compliance(&path) {
                out.push(TeamReportLink {
                    kind: "compliance".to_string(),
                    path: path.display().to_string(),
                });
            }
        }
    }
    Ok(out)
}

fn is_compliance(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("compliance-"))
}

fn totals(projects: &[TeamProject]) -> TeamTotals {
    TeamTotals {
        projects: projects.iter().filter(|project| project.exists).count(),
        transactions: projects.iter().map(|project| project.transactions).sum(),
        approvals: projects.iter().map(|project| project.approvals.total).sum(),
        audit_events: projects.iter().map(|project| project.audit_events).sum(),
        remote_runners: projects
            .iter()
            .map(|project| project.runners.remote_count)
            .sum(),
        total_cost_usd: projects
            .iter()
            .map(|project| analytics_cost(&project.analytics))
            .sum(),
    }
}

fn analytics_cost(summary: &AnalyticsSummary) -> f64 {
    summary
        .by_task_type
        .iter()
        .map(|bucket| bucket.total_cost_usd)
        .sum()
}
