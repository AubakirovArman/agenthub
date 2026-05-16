use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;
use std::{env, fs};

use anyhow::Result;

use crate::spec::AgentSpec;

pub(super) enum Decision {
    Run,
    Cancel,
}

pub(super) fn confirm_plan(spec_path: &Path) -> Result<Decision> {
    let mut spec = AgentSpec::load(spec_path)?;
    print_plan(&spec);
    if !io::stdin().is_terminal() {
        return Ok(Decision::Run);
    }
    loop {
        print!("Run this transaction? [Y/n/diff/details/edit] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        match line.trim().to_ascii_lowercase().as_str() {
            "" | "y" | "yes" | "д" | "да" => {
                AgentSpec::load(spec_path)?;
                return Ok(Decision::Run);
            }
            "n" | "no" | "н" | "нет" => return Ok(Decision::Cancel),
            "details" | "d" | "детали" => println!("{}", fs::read_to_string(spec_path)?),
            "diff" | "patch" | "патч" => print_diff_preview(spec_path, &spec),
            "edit" | "e" | "редактировать" => {
                open_editor(spec_path)?;
                match AgentSpec::load(spec_path) {
                    Ok(updated) => {
                        spec = updated;
                        print_plan(&spec);
                    }
                    Err(error) => {
                        println!("edited spec is invalid: {error}");
                        println!("Use edit again, details, cancel, or fix the file manually.");
                    }
                }
            }
            _ => println!("Use Y, n, diff, details, or edit."),
        }
    }
}

fn print_plan(spec: &AgentSpec) {
    println!("Plan:");
    println!(
        "- task: {}",
        spec.task.title.as_deref().unwrap_or(&spec.task.id)
    );
    println!(
        "- target: {}",
        spec.task.target.as_deref().unwrap_or("<none>")
    );
    println!(
        "- provider: {}",
        spec.agent.adapter.as_deref().unwrap_or("command")
    );
    println!(
        "- verifier: {}",
        spec.verify.profile.as_deref().unwrap_or("default")
    );
    println!("- scope allow: {}", list_or_none(&spec.scope.allow));
    println!("- scope deny: {}", list_or_none(&spec.scope.deny));
    let commands = all_commands(spec);
    if !commands.is_empty() {
        println!("- commands: {}", commands.join(" && "));
    }
    let (level, reason) = risk_summary(spec);
    println!("- risk: {level} - {reason}");
}

fn print_diff_preview(spec_path: &Path, spec: &AgentSpec) {
    println!("Diff preview:");
    println!("- no generated patch exists before execution");
    println!("- draft: {}", spec_path.display());
    println!("- allowed paths: {}", list_or_none(&spec.scope.allow));
    println!("- denied paths: {}", list_or_none(&spec.scope.deny));
    println!("- run `/diff` after execution to inspect actual changes");
}

fn open_editor(spec_path: &Path) -> Result<()> {
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| default_editor().to_string());
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or(default_editor());
    let status = Command::new(program).args(parts).arg(spec_path).status()?;
    if status.success() {
        println!("edited {}", spec_path.display());
    } else {
        println!("editor exited with status {status}");
    }
    Ok(())
}

fn default_editor() -> &'static str {
    if cfg!(windows) {
        "notepad"
    } else {
        "vi"
    }
}

fn all_commands(spec: &AgentSpec) -> Vec<String> {
    spec.execution
        .commands
        .iter()
        .chain(spec.verify.commands.iter())
        .chain(spec.review.commands.iter())
        .chain(spec.repair.commands.iter())
        .cloned()
        .collect()
}

fn risk_summary(spec: &AgentSpec) -> (&'static str, String) {
    let commands = all_commands(spec);
    if commands.iter().any(|command| is_block_like(command)) {
        return (
            "high",
            "destructive or privileged command is present".to_string(),
        );
    }
    if commands.iter().any(|command| is_dependency_change(command)) {
        return (
            "medium",
            "dependency or package command needs attention".to_string(),
        );
    }
    if spec
        .scope
        .allow
        .iter()
        .any(|item| matches!(item.as_str(), "*" | "**"))
    {
        return ("medium", "scope allows broad workspace edits".to_string());
    }
    if spec.transaction.approval_required {
        return (
            "medium",
            "spec explicitly requires human approval".to_string(),
        );
    }
    (
        "low",
        "bounded scope and verifier-controlled execution".to_string(),
    )
}

fn is_block_like(command: &str) -> bool {
    let command = command.to_ascii_lowercase();
    command.contains("rm -rf")
        || command.contains("sudo ")
        || command.contains("terraform apply")
        || command.contains("kubectl delete")
        || command.contains("chmod -r 777")
}

fn is_dependency_change(command: &str) -> bool {
    let command = command.to_ascii_lowercase();
    [
        "npm install",
        "pnpm add",
        "yarn add",
        "cargo add",
        "pip install",
    ]
    .iter()
    .any(|needle| {
        command
            .lines()
            .map(str::trim_start)
            .any(|line| command_starts_with(line, needle))
    })
}

fn command_starts_with(line: &str, needle: &str) -> bool {
    line == needle
        || line
            .strip_prefix(needle)
            .is_some_and(|rest| rest.starts_with(' '))
        || line.contains(&format!("&& {needle}"))
        || line.contains(&format!("; {needle}"))
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "<none>".to_string()
    } else {
        items.join(", ")
    }
}
