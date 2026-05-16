use std::io::{self, IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::product_cli::bootstrap;
use crate::product_cli::{config, providers};
use crate::{agent_dir, git};

pub(super) fn prepare(root: &Path) -> Result<()> {
    println!("AgentHub {}", crate::product_cli::version());
    println!("Working folder: {}", root.display());
    ensure_git(root)?;
    ensure_agent(root)?;
    ensure_baseline(root)?;
    suggest_provider(root)?;
    println!("Type a task. Use / for commands, /cd <folder> to switch projects.");
    Ok(())
}

fn ensure_git(root: &Path) -> Result<()> {
    std::fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    if git::is_repo(root) {
        return Ok(());
    }
    println!("This folder is not a Git repository.");
    if confirm("Git repo not found. Create git repo here?", true)? {
        git::init(root)?;
        println!("Git: initialized");
    } else {
        println!("Git: skipped; transactions need a git repository");
    }
    Ok(())
}

fn ensure_baseline(root: &Path) -> Result<()> {
    if git::has_head(root) {
        return Ok(());
    }
    if confirm(
        "Initial commit not found. Create AgentHub baseline commit?",
        true,
    )? {
        if bootstrap::ensure_baseline(root)? {
            println!("Git: baseline committed");
        }
    } else {
        println!("Git: baseline skipped; transactions need an initial commit");
    }
    Ok(())
}

fn ensure_agent(root: &Path) -> Result<()> {
    if root.join(".agent/project.yaml").exists() {
        return Ok(());
    }
    println!("This folder has no .agent runtime directory.");
    if confirm(".agent not found. Initialize AgentHub project?", true)? {
        agent_dir::init_project(root, false)?;
        println!(".agent: initialized");
    } else {
        println!(".agent: skipped; reports, memory, and dashboard need initialization");
    }
    Ok(())
}

fn suggest_provider(root: &Path) -> Result<()> {
    let current = config::default_provider(root)?;
    if current != "command" || config::path(root).exists() {
        println!("Provider: {current}  (change with /providers)");
        return Ok(());
    }
    if !io::stdin().is_terminal() {
        println!("Provider: command  (change with /providers)");
        return Ok(());
    }
    let preferred = providers::statuses(root)?.into_iter().find(|status| {
        status.available && matches!(status.info.id.as_str(), "codex" | "kimi" | "gemini")
    });
    if let Some(status) = preferred {
        if confirm(
            &format!("Use {} as default provider?", status.info.id),
            true,
        )? {
            print!("{}", providers::setup_provider(root, &status.info.id)?);
            return Ok(());
        }
    }
    println!("Provider: command  (change with /providers)");
    Ok(())
}

fn confirm(question: &str, default_yes: bool) -> Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(default_yes);
    }
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{question} {suffix} ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let answer = line.trim().to_ascii_lowercase();
    Ok(match answer.as_str() {
        "" => default_yes,
        "y" | "yes" | "д" | "да" => true,
        _ => false,
    })
}
