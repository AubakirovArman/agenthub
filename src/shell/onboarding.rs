use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::product_cli::{config, providers};
use crate::{agent_dir, git};

pub(super) fn prepare(root: &Path) -> Result<()> {
    println!("AgentHub {}", crate::product_cli::version());
    println!("Project: {}", root.display());
    ensure_git(root)?;
    ensure_agent(root)?;
    suggest_provider(root)?;
    println!("Type what you want to build, fix, inspect, or change.");
    println!("Use / for commands, @ for files, ! for shell, # for memory.");
    Ok(())
}

fn ensure_git(root: &Path) -> Result<()> {
    std::fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    if git::is_repo(root) {
        println!("Git: ok");
        return Ok(());
    }
    if confirm("Git repo not found. Create git repo here?", true)? {
        let output = Command::new("git")
            .arg("init")
            .current_dir(root)
            .output()
            .with_context(|| format!("run git init in {}", root.display()))?;
        if !output.status.success() {
            return Err(anyhow!(
                "git init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        println!("Git: initialized");
    } else {
        println!("Git: skipped; transactions need a git repository");
    }
    Ok(())
}

fn ensure_agent(root: &Path) -> Result<()> {
    if root.join(".agent/project.yaml").exists() {
        println!(".agent: ok");
        return Ok(());
    }
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
        println!("Provider: {current}");
        return Ok(());
    }
    if !io::stdin().is_terminal() {
        println!("Provider: command");
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
    println!("Provider: command");
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
