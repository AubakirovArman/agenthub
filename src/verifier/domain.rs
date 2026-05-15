mod backend;
mod common;
mod profiles;
mod research;

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerificationResult {
    pub profile: String,
    pub passed: bool,
    pub checks: Vec<DomainCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCheckResult {
    pub name: String,
    pub success: bool,
    pub detail: String,
}

pub fn run(profile: Option<&str>, worktree: &Path) -> Result<Option<DomainVerificationResult>> {
    let Some(profile) = profile else {
        return Ok(None);
    };
    let checks = match profile {
        "content_quality" => profiles::content_checks(worktree)?,
        "data_quality" => profiles::data_checks(worktree)?,
        "infra_plan" => profiles::infra_checks(worktree)?,
        "media_render" => profiles::media_checks(worktree)?,
        "backend_tdd" => backend::backend_checks(worktree)?,
        "research_report" => research::research_checks(worktree)?,
        _ => return Ok(None),
    };
    Ok(Some(DomainVerificationResult {
        profile: profile.to_string(),
        passed: checks.iter().all(|check| check.success),
        checks,
    }))
}
