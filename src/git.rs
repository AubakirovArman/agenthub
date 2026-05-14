use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
}

pub fn is_repo(root: &Path) -> bool {
    run(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|output| output.stdout.trim() == "true")
        .unwrap_or(false)
}

pub fn head(root: &Path) -> Result<Option<String>> {
    match run(root, &["rev-parse", "HEAD"]) {
        Ok(output) => Ok(Some(output.stdout.trim().to_string())),
        Err(_) => Ok(None),
    }
}

pub fn current_branch(root: &Path) -> Result<String> {
    let output = run(root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    Ok(output.stdout.trim().to_string())
}

pub fn status_porcelain(root: &Path) -> Result<String> {
    Ok(run(root, &["status", "--porcelain"])?.stdout)
}

pub fn dirty(root: &Path) -> bool {
    dirty_blockers(root)
        .map(|blockers| !blockers.is_empty())
        .unwrap_or(true)
}

pub fn dirty_blockers(root: &Path) -> Result<Vec<String>> {
    let status = status_porcelain(root)?;
    let mut blockers = Vec::new();
    for line in status.lines() {
        if let Some(path) = status_path(line) {
            if !is_runtime_agent_path(&path) {
                blockers.push(path);
            }
        }
    }
    Ok(blockers)
}

pub fn create_worktree(root: &Path, branch: &str, path: &Path) -> Result<()> {
    let path_arg = path.to_string_lossy().to_string();
    run(
        root,
        &["worktree", "add", "-b", branch, &path_arg],
    )
    .map(|_| ())
}

pub fn remove_worktree(root: &Path, path: &Path) -> Result<()> {
    let path_arg = path.to_string_lossy().to_string();
    run(root, &["worktree", "remove", "--force", &path_arg])
    .map(|_| ())
}

pub fn add_all(root: &Path) -> Result<()> {
    run(root, &["add", "-A"]).map(|_| ())
}

pub fn commit(root: &Path, message: &str) -> Result<bool> {
    if status_porcelain(root)?.trim().is_empty() {
        return Ok(false);
    }
    run(root, &["commit", "-m", message])?;
    Ok(true)
}

pub fn merge_ff_only(root: &Path, branch: &str) -> Result<()> {
    run(root, &["merge", "--ff-only", branch]).map(|_| ())
}

pub fn diff_numstat(root: &Path) -> Result<String> {
    Ok(run(root, &["diff", "--numstat", "HEAD"])
        .unwrap_or_else(|_| GitOutput {
            stdout: String::new(),
            stderr: String::new(),
        })
        .stdout)
}

pub fn changed_files(root: &Path) -> Result<Vec<String>> {
    let status = status_porcelain(root)?;
    let mut files = Vec::new();
    for line in status.lines() {
        if let Some(path) = status_path(line) {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn status_path(line: &str) -> Option<String> {
    if line.len() < 4 {
        return None;
    }
    let raw = line[3..].trim();
    Some(
        raw.split_once(" -> ")
            .map(|(_, after)| after)
            .unwrap_or(raw)
            .to_string(),
    )
}

fn is_runtime_agent_path(path: &str) -> bool {
    path.starts_with(".agent/tx/")
        || path.starts_with(".agent/workspaces/")
        || path.starts_with(".agent/cache/")
        || path == ".agent/memory/committed.jsonl"
        || path == ".agent/memory/failed_attempts.jsonl"
}

pub fn run(root: &Path, args: &[&str]) -> Result<GitOutput> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("run git {} in {}", args.join(" "), root.display()))?;

    if !output.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
