use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

mod history;

pub use history::{find_commit_by_subject, revert_no_edit};

#[derive(Debug, Clone)]
pub struct GitOutput {
    pub stdout: String,
}

pub fn is_repo(root: &Path) -> bool {
    run(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|output| output.stdout.trim() == "true")
        .unwrap_or(false)
}

pub fn init(root: &Path) -> Result<()> {
    run(root, &["init"]).map(|_| ())
}

pub fn head(root: &Path) -> Result<Option<String>> {
    match run(root, &["rev-parse", "HEAD"]) {
        Ok(output) => Ok(Some(output.stdout.trim().to_string())),
        Err(_) => Ok(None),
    }
}

pub fn has_head(root: &Path) -> bool {
    head(root).ok().flatten().is_some()
}

pub fn ensure_identity(root: &Path) -> Result<()> {
    if run(root, &["config", "user.email"]).is_err() {
        run(root, &["config", "user.email", "agenthub@example.invalid"])?;
    }
    if run(root, &["config", "user.name"]).is_err() {
        run(root, &["config", "user.name", "AgentHub"])?;
    }
    Ok(())
}

pub fn current_branch(root: &Path) -> Result<String> {
    let output = run(root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    Ok(output.stdout.trim().to_string())
}

pub fn status_porcelain(root: &Path) -> Result<String> {
    Ok(run(root, &["status", "--porcelain", "--untracked-files=all"])?.stdout)
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
    run(root, &["worktree", "add", "-b", branch, &path_arg]).map(|_| ())
}

pub fn remove_worktree(root: &Path, path: &Path) -> Result<()> {
    let path_arg = path.to_string_lossy().to_string();
    run(root, &["worktree", "remove", "--force", &path_arg]).map(|_| ())
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

pub fn rebase_onto(root: &Path, new_base: &str) -> Result<()> {
    run(root, &["rebase", "--autostash", new_base]).map(|_| ())
}

pub fn changed_files_between(root: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    let output = run(root, &["diff", "--name-only", base, head])?;
    Ok(sorted_lines(&output.stdout))
}

pub fn tracked_files(root: &Path) -> Result<Vec<String>> {
    let output = run(root, &["ls-files"])?;
    Ok(sorted_lines(&output.stdout))
}

pub fn diff_numstat(root: &Path) -> Result<String> {
    Ok(run(root, &["diff", "--numstat", "HEAD"])
        .unwrap_or_else(|_| GitOutput {
            stdout: String::new(),
        })
        .stdout)
}

fn sorted_lines(value: &str) -> Vec<String> {
    let mut files = value
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
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
        || path.starts_with(".agent/drafts/")
        || path.starts_with(".agent/shell/")
        || path.starts_with(".agent/reports/")
        || path.starts_with(".agent/metrics/")
        || path.starts_with(".agent/memory/compacted/")
        || path.starts_with(".agent/memory/views/")
        || path == ".agent/config.yaml"
        || path == ".agent/enterprise/audit.jsonl"
        || path == ".agent/memory/audit.json"
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
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn dirty_blockers_ignore_local_agent_config() -> Result<()> {
        let dir = tempfile::tempdir()?;
        init(dir.path())?;
        fs::create_dir_all(dir.path().join(".agent"))?;
        fs::write(
            dir.path().join(".agent/config.yaml"),
            "default_provider: codex\n",
        )?;

        assert!(dirty_blockers(dir.path())?.is_empty());
        Ok(())
    }
}
