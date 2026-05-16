use std::fs;
use std::path::Path;

use crate::{agent_dir, git, home, product_cli, workspace};

use super::chat::ChatSession;
use super::format;

pub(super) fn render(root: &Path, session: &ChatSession, tx: Option<&str>) -> String {
    let project = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project");
    let mode = prompt_mode(root, session);
    let (provider, provider_ready) = display_provider(root, mode.as_str());
    let git_state = if !home::project_has_runtime(root) {
        "git optional"
    } else if git::is_repo(root) {
        if git::dirty(root) {
            "git ~"
        } else {
            "git ok"
        }
    } else {
        "git missing"
    };
    let tx_label = tx
        .map(|id| tx_status(root, id).unwrap_or_else(|| format!("tx {}", short_tx(id))))
        .or_else(|| latest_tx(root));
    let approvals = pending_approvals(root);
    let provider_label = if provider_ready {
        format::styled(&format!("{provider} ok"), format::Color::Green)
    } else {
        format::styled(&format!("{provider} warn"), format::Color::Yellow)
    };
    let mut context = format!(
        "{} | {} | {provider_label} | {git_state}",
        project,
        mode.as_str()
    );
    if let Some(tx_label) = tx_label {
        context.push_str(" | ");
        context.push_str(&tx_label);
    }
    if approvals > 0 {
        context.push_str(" | approvals ");
        context.push_str(&approvals.to_string());
    }
    format!(
        "{}agenthub{} {}({}){}> ",
        format::bold_color(format::Color::Green),
        format::reset(),
        format::color(format::Color::DarkGray),
        context,
        format::reset()
    )
}

fn prompt_mode(root: &Path, session: &ChatSession) -> workspace::WorkspaceMode {
    if home::project_has_runtime(root) {
        return workspace::WorkspaceMode::Project;
    }
    super::chat::latest_intent_mode(session)
        .ok()
        .flatten()
        .as_deref()
        .and_then(|mode| match mode {
            "ops" => Some(workspace::WorkspaceMode::Ops),
            "chat" => Some(workspace::WorkspaceMode::Chat),
            _ => None,
        })
        .unwrap_or_else(|| workspace::detect_mode(root).mode)
}

fn display_provider(root: &Path, mode: &str) -> (String, bool) {
    let default = product_cli::config::default_provider(root)
        .unwrap_or_else(|_| product_cli::config::DEFAULT_PROVIDER.to_string());
    let statuses = product_cli::providers::statuses(root).unwrap_or_default();
    if mode == "chat" {
        if let Some(status) = statuses.iter().find(|status| {
            status.info.id == default
                && matches!(status.info.id.as_str(), "deepseek" | "kimi")
                && status.available
        }) {
            return (status.info.id.clone(), true);
        }
        if let Some(status) = statuses.iter().find(|status| {
            matches!(status.info.id.as_str(), "deepseek" | "kimi") && status.available
        }) {
            return (status.info.id.clone(), true);
        }
    }
    let ready = statuses
        .into_iter()
        .find(|status| status.info.id == default)
        .is_some_and(|status| status.available);
    if matches!(default.as_str(), "deepseek" | "kimi") {
        (default, ready)
    } else {
        (product_cli::config::DEFAULT_PROVIDER.to_string(), false)
    }
}

fn short_tx(id: &str) -> String {
    id.strip_prefix("tx-")
        .unwrap_or(id)
        .chars()
        .take(8)
        .collect()
}

fn tx_status(root: &Path, tx_id: &str) -> Option<String> {
    let row = agent_dir::list_transactions(root)
        .ok()?
        .into_iter()
        .find(|row| row.id == tx_id)?;
    Some(format!(
        "{} {}",
        short_tx(&row.id),
        format::status_label(&row.status)
    ))
}

fn latest_tx(root: &Path) -> Option<String> {
    let row = agent_dir::list_transactions(root).ok()?.pop()?;
    Some(format!(
        "{} {}",
        short_tx(&row.id),
        format::status_label(&row.status)
    ))
}

fn pending_approvals(root: &Path) -> usize {
    agent_dir::list_transactions(root)
        .unwrap_or_default()
        .into_iter()
        .filter(|row| row.status == "BLOCKED_ON_HUMAN")
        .count()
        + pending_specs(root)
}

fn pending_specs(root: &Path) -> usize {
    let specs = root.join(".agent/specs");
    if !specs.exists() {
        return 0;
    }
    fs::read_dir(specs)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter(|entry| {
            fs::read_to_string(entry.path())
                .unwrap_or_default()
                .contains("approval_required: true")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::agent_dir;

    use super::*;

    #[test]
    fn prompt_includes_project_provider_and_git_context() -> Result<()> {
        let dir = tempfile::tempdir()?;
        agent_dir::init_project(dir.path(), false)?;
        let chat = crate::shell::chat::create(dir.path())?;

        let prompt = render(dir.path(), &chat, None);

        assert!(prompt.contains("agenthub"));
        assert!(prompt.contains("deepseek"));
        assert!(prompt.contains("git"));
        Ok(())
    }

    #[test]
    fn prompt_surfaces_latest_ops_mode_without_project_runtime() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::create_dir_all(dir.path().join(".agent/shell"))?;
        let chat = crate::shell::chat::create(dir.path())?;
        crate::shell::chat::append_intent(
            &chat,
            "ops_advice",
            "ops",
            "check server load",
            "server or operations wording without project runtime",
        )?;

        let prompt = render(dir.path(), &chat, None);

        assert!(prompt.contains("ops"));
        Ok(())
    }
}
