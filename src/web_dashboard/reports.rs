use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::agent_dir::{AgentPaths, TransactionRow};
use crate::web_dashboard::read::file_href;
use crate::web_dashboard::ReportLink;

pub fn collect_reports(project_root: &Path, rows: &[TransactionRow]) -> Result<Vec<ReportLink>> {
    let mut reports = rows
        .iter()
        .rev()
        .take(10)
        .map(|row| ReportLink {
            title: row.id.clone(),
            kind: "transaction".to_string(),
            href: file_href(&row.report_path),
        })
        .collect::<Vec<_>>();
    reports.extend(compliance_reports(project_root)?);
    Ok(reports)
}

fn compliance_reports(project_root: &Path) -> Result<Vec<ReportLink>> {
    let dir = AgentPaths::new(project_root).enterprise;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let path = entry?.path();
        if is_compliance_report(&path) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths.into_iter().rev().take(5).map(report_link).collect())
}

fn is_compliance_report(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .starts_with("compliance-")
}

fn report_link(path: PathBuf) -> ReportLink {
    ReportLink {
        title: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("report.md")
            .to_string(),
        kind: "compliance".to_string(),
        href: file_href(&path),
    }
}
