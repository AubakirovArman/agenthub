use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::{agent_dir, git, memory, product_cli, skill_registry};

use super::format;

pub(super) fn print(root: &Path) -> Result<()> {
    let project = project_name(root);
    let project_type = project_type(root);
    let default_provider = crate::product_cli::config::default_provider(root)?;
    let provider_ready = crate::product_cli::providers::statuses(root)?
        .into_iter()
        .find(|status| status.info.id == default_provider)
        .is_some_and(|status| status.available);
    let skills = skill_registry::list_available(root)
        .map(|items| items.len())
        .unwrap_or(0);
    let memory = memory::inspect(root).ok();
    let rows = agent_dir::list_transactions(root).unwrap_or_default();

    println!();
    println!(
        "  {}AgentHub v{}{}",
        format::bold_color(format::Color::Cyan),
        product_cli::version(),
        format::reset()
    );
    println!("  Project: {project} ({project_type})");
    println!(
        "  Provider: {}  Git: {}  .agent: {}",
        if provider_ready {
            format::styled(&format!("{default_provider} ready"), format::Color::Green)
        } else {
            format::styled(
                &format!("{default_provider} limited"),
                format::Color::Yellow,
            )
        },
        if git::is_repo(root) { "ok" } else { "missing" },
        if root.join(".agent/project.yaml").exists() {
            "ok"
        } else {
            "missing"
        }
    );
    println!(
        "  Skills: {} loaded  Memory: {} records",
        skills,
        memory.map(|stats| stats.committed).unwrap_or(0)
    );
    println!();
    println!("  Quick start:");
    println!("  - Type a request: \"add a health check\"");
    println!("  - /help for commands  @file.rs for context  !command for shell");
    if !rows.is_empty() {
        println!();
        println!("  Recent:");
        for row in rows.iter().rev().take(3) {
            println!("  - {}  {}", row.id, format::status_label(&row.status));
        }
    }
    let pending = rows
        .iter()
        .filter(|row| row.status == "BLOCKED_ON_HUMAN")
        .count()
        + pending_specs(root)?;
    if pending > 0 {
        println!();
        format::warning(&format!(
            "{pending} approval item(s) pending. Type /approvals."
        ));
    }
    println!();
    Ok(())
}

fn project_name(root: &Path) -> String {
    root.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project")
        .to_string()
}

fn project_type(root: &Path) -> &'static str {
    if root.join("Cargo.toml").exists() {
        "Rust"
    } else if root.join("package.json").exists() {
        "Node"
    } else if root.join("pyproject.toml").exists() {
        "Python"
    } else {
        "workspace"
    }
}

fn pending_specs(root: &Path) -> Result<usize> {
    let specs = root.join(".agent/specs");
    if !specs.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(specs)? {
        let path = entry?.path();
        if path.is_file()
            && fs::read_to_string(&path)
                .unwrap_or_default()
                .contains("approval_required: true")
        {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::project_type;

    #[test]
    fn detects_project_type_for_welcome() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='x'\n")?;
        assert_eq!(project_type(dir.path()), "Rust");
        Ok(())
    }
}
