use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{chat_index, git};

use super::format;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Checkpoint {
    name: String,
    created_at: DateTime<Utc>,
    git_commit: String,
    current_tx: Option<String>,
    current_chat: Option<String>,
}

pub(super) fn show_rewind(root: &Path) -> Result<()> {
    format::section("Session History");
    let rows = chat_index::list(root, 20)?;
    if rows.is_empty() {
        println!("  no chat sessions yet");
        return Ok(());
    }
    for (index, row) in rows.iter().enumerate() {
        println!(
            "  [{:>2}] {}  {}  messages:{} tx:{}",
            index + 1,
            row.title,
            format::muted(&row.id),
            row.messages,
            row.txs
        );
    }
    println!();
    println!("  Open a session with /chat <id>. Selective git rewind is not automatic.");
    Ok(())
}

pub(super) fn save_checkpoint(
    root: &Path,
    name: &str,
    current_tx: Option<&str>,
    current_chat: Option<&str>,
) -> Result<()> {
    let name = sanitize_name(name)?;
    let git_commit =
        git::head(root)?.ok_or_else(|| anyhow!("cannot save checkpoint without HEAD"))?;
    let checkpoint = Checkpoint {
        name: name.clone(),
        created_at: Utc::now(),
        git_commit,
        current_tx: current_tx.map(str::to_string),
        current_chat: current_chat.map(str::to_string),
    };
    let path = checkpoint_path(root, &name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(&checkpoint)?)
        .with_context(|| format!("write {}", path.display()))?;
    format::success(&format!("Checkpoint `{name}` saved at {}", path.display()));
    Ok(())
}

pub(super) fn restore_checkpoint(root: &Path, name: &str) -> Result<()> {
    let name = sanitize_name(name)?;
    let checkpoint = load_checkpoint(root, &name)?;
    let blockers = git::dirty_blockers(root)?;
    if !blockers.is_empty() {
        return Err(anyhow!(
            "cannot restore checkpoint with dirty worktree; commit or stash first: {}",
            blockers.join(", ")
        ));
    }
    git::run(root, &["checkout", &checkpoint.git_commit, "--", "."])?;
    format::success(&format!(
        "Restored checkpoint `{}` at commit {}",
        checkpoint.name,
        short(&checkpoint.git_commit)
    ));
    if let Some(chat) = checkpoint.current_chat {
        println!("  chat: {chat}");
    }
    if let Some(tx) = checkpoint.current_tx {
        println!("  tx: {tx}");
    }
    Ok(())
}

fn load_checkpoint(root: &Path, name: &str) -> Result<Checkpoint> {
    let path = checkpoint_path(root, name);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn checkpoint_path(root: &Path, name: &str) -> PathBuf {
    root.join(".agent")
        .join("shell")
        .join("checkpoints")
        .join(format!("{name}.json"))
}

fn sanitize_name(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(anyhow!("checkpoint name is required"));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(anyhow!(
            "checkpoint name may contain only ASCII letters, digits, dash, underscore, or dot"
        ));
    }
    Ok(value.to_string())
}

fn short(value: &str) -> String {
    value.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn saves_checkpoint_metadata() -> Result<()> {
        let dir = tempfile::tempdir()?;
        git::init(dir.path())?;
        git::ensure_identity(dir.path())?;
        fs::write(dir.path().join("README.md"), "demo")?;
        git::add_all(dir.path())?;
        assert!(git::commit(dir.path(), "initial")?);

        save_checkpoint(dir.path(), "before-auth", Some("tx-1"), Some("chat-1"))?;

        let text =
            fs::read_to_string(dir.path().join(".agent/shell/checkpoints/before-auth.json"))?;
        assert!(text.contains("before-auth"));
        assert!(text.contains("tx-1"));
        Ok(())
    }
}
