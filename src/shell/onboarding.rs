use std::path::Path;

use anyhow::{Context, Result};

use crate::product_cli::{config, providers};
use crate::{git, workspace};

pub(super) fn prepare(root: &Path) -> Result<()> {
    println!("AgentHub {}", crate::product_cli::version());
    println!("Working folder: {}", root.display());
    std::fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    print_mode(root);
    suggest_provider(root)?;
    println!("Type a message. Use / for commands, /cd <folder> to switch projects.");
    Ok(())
}

fn suggest_provider(root: &Path) -> Result<()> {
    let current = config::default_provider(root)?;
    let statuses = providers::statuses(root)?;
    if let Some(status) = statuses
        .iter()
        .find(|status| status.info.id == current && status.available)
    {
        println!(
            "Provider: {} ready  (change with /providers)",
            status.info.id
        );
        return Ok(());
    }
    let preferred = statuses.into_iter().find(|status| status.available);
    if let Some(status) = preferred {
        println!(
            "Provider: {} ready  (change with /providers)",
            status.info.id
        );
        return Ok(());
    }
    println!(
        "Provider: {} missing  (configure DeepSeek/Kimi with /providers)",
        config::DEFAULT_PROVIDER
    );
    Ok(())
}

fn print_mode(root: &Path) {
    match workspace::detect_mode(root).mode {
        workspace::WorkspaceMode::Project => {
            println!(
                "Mode: project  Git: {}  .agent: ok",
                if git::is_repo(root) { "ok" } else { "missing" }
            );
        }
        workspace::WorkspaceMode::Chat | workspace::WorkspaceMode::Ops if git::is_repo(root) => {
            println!("Mode: chat  Git: detected  .agent: not initialized");
            println!("Project transactions are available after /init or `agenthub run ...`.");
        }
        workspace::WorkspaceMode::Chat | workspace::WorkspaceMode::Ops => {
            println!("Mode: chat  Git: not required  .agent: not required");
        }
    }
}
