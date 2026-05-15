use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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
        "content_quality" => content_checks(worktree)?,
        "data_quality" => data_checks(worktree)?,
        "infra_plan" => infra_checks(worktree)?,
        _ => return Ok(None),
    };
    Ok(Some(DomainVerificationResult {
        profile: profile.to_string(),
        passed: checks.iter().all(|check| check.success),
        checks,
    }))
}

fn content_checks(root: &Path) -> Result<Vec<DomainCheckResult>> {
    let files = collect_files(&root.join("content"), &["md", "txt"])?;
    Ok(vec![
        present("content_files_present", &files),
        all_non_empty("content_files_non_empty", &files)?,
    ])
}

fn data_checks(root: &Path) -> Result<Vec<DomainCheckResult>> {
    let files = collect_files(&root.join("data"), &["json"])?;
    Ok(vec![
        present("data_json_present", &files),
        all_json("data_json_valid", &files)?,
    ])
}

fn infra_checks(root: &Path) -> Result<Vec<DomainCheckResult>> {
    let files = collect_files(&root.join("infra"), &["yaml", "yml", "tf"])?;
    Ok(vec![
        present("infra_artifacts_present", &files),
        infra_artifacts_valid("infra_artifacts_valid", &files)?,
    ])
}

fn present(name: &str, files: &[PathBuf]) -> DomainCheckResult {
    check(name, !files.is_empty(), format!("{} file(s)", files.len()))
}

fn all_non_empty(name: &str, files: &[PathBuf]) -> Result<DomainCheckResult> {
    let empty = files
        .iter()
        .filter(|file| {
            fs::metadata(file)
                .map(|meta| meta.len() == 0)
                .unwrap_or(true)
        })
        .count();
    Ok(check(
        name,
        empty == 0 && !files.is_empty(),
        format!("{empty} empty"),
    ))
}

fn all_json(name: &str, files: &[PathBuf]) -> Result<DomainCheckResult> {
    let mut invalid = 0;
    for file in files {
        let content =
            fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
        if serde_json::from_str::<serde_json::Value>(&content).is_err() {
            invalid += 1;
        }
    }
    Ok(check(
        name,
        invalid == 0 && !files.is_empty(),
        format!("{invalid} invalid"),
    ))
}

fn infra_artifacts_valid(name: &str, files: &[PathBuf]) -> Result<DomainCheckResult> {
    let mut invalid = 0;
    for file in files {
        let content =
            fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
        if content.trim().is_empty() || yaml_invalid(file, &content) {
            invalid += 1;
        }
    }
    Ok(check(
        name,
        invalid == 0 && !files.is_empty(),
        format!("{invalid} invalid"),
    ))
}

fn yaml_invalid(file: &Path, content: &str) -> bool {
    matches!(
        file.extension().and_then(|ext| ext.to_str()),
        Some("yaml" | "yml")
    ) && serde_yaml::from_str::<serde_yaml::Value>(content).is_err()
}

fn collect_files(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit(root, extensions, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            visit(&path, extensions, files)?;
        } else if has_extension(&path, extensions) {
            files.push(path);
        }
    }
    Ok(())
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.contains(&ext))
}

fn check(name: &str, success: bool, detail: String) -> DomainCheckResult {
    DomainCheckResult {
        name: name.to_string(),
        success,
        detail,
    }
}
