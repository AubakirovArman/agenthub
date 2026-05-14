use std::path::Path;

use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

use crate::git;
use crate::spec::{DiffLimitsSpec, ScopeSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffGuardResult {
    pub passed: bool,
    pub summary: DiffSummary,
    pub violations: Vec<String>,
}

pub fn check(worktree: &Path, scope: &ScopeSpec, limits: &DiffLimitsSpec) -> Result<DiffGuardResult> {
    let changed_files = git::changed_files(worktree)?;
    let (lines_added, lines_deleted) = count_numstat(worktree)?;
    let summary = DiffSummary {
        files_changed: changed_files.len(),
        lines_added,
        lines_deleted,
        changed_files: changed_files.clone(),
    };

    let mut violations = Vec::new();
    if summary.files_changed > limits.max_files_changed {
        violations.push(format!(
            "files changed {} exceeds limit {}",
            summary.files_changed, limits.max_files_changed
        ));
    }
    if summary.lines_added > limits.max_lines_added {
        violations.push(format!(
            "lines added {} exceeds limit {}",
            summary.lines_added, limits.max_lines_added
        ));
    }
    if summary.lines_deleted > limits.max_lines_deleted {
        violations.push(format!(
            "lines deleted {} exceeds limit {}",
            summary.lines_deleted, limits.max_lines_deleted
        ));
    }

    let allow = compile_globs(&scope.allow).context("compile allow scope")?;
    let deny = compile_globs(&scope.deny).context("compile deny scope")?;
    for file in &changed_files {
        if let Some(deny) = &deny {
            if deny.is_match(file) {
                violations.push(format!("out-of-policy denied path changed: {file}"));
            }
        }
        if let Some(allow) = &allow {
            if !allow.is_match(file) {
                violations.push(format!("path is outside allowed scope: {file}"));
            }
        }
    }

    Ok(DiffGuardResult {
        passed: violations.is_empty(),
        summary,
        violations,
    })
}

fn count_numstat(worktree: &Path) -> Result<(usize, usize)> {
    let mut added = 0;
    let mut deleted = 0;
    for line in git::diff_numstat(worktree)?.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        added += parse_numstat(parts[0]);
        deleted += parse_numstat(parts[1]);
    }
    Ok((added, deleted))
}

fn parse_numstat(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(0)
}

fn compile_globs(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern).map_err(|error| anyhow!("invalid glob `{pattern}`: {error}"))?);
    }
    Ok(Some(builder.build()?))
}

