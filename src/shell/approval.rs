use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;
use std::{env, fs};

use anyhow::Result;

use crate::spec::AgentSpec;

use super::{format, inline_approval};

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
        print!("Action [Enter=run/e=edit/d=draft/v=verify/x=effects/q=cancel] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        match line.trim().to_ascii_lowercase().as_str() {
            "" | "y" | "yes" | "д" | "да" => {
                AgentSpec::load(spec_path)?;
                return Ok(Decision::Run);
            }
            "n" | "no" | "q" | "cancel" | "н" | "нет" => return Ok(Decision::Cancel),
            "details" | "d" | "draft" | "детали" => {
                println!(
                    "{}",
                    format::code_block("yaml", &fs::read_to_string(spec_path)?)
                )
            }
            "v" | "verify" => print_verify_details(&spec),
            "x" | "effects" | "diff" | "patch" | "патч" => print_diff_preview(spec_path, &spec),
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
            _ => println!("Use Enter, e, d, v, x, or q."),
        }
    }
}

fn print_plan(spec: &AgentSpec) {
    print!("{}", render_plan_card(spec, None));
}

fn render_plan_card(spec: &AgentSpec, spec_path: Option<&Path>) -> String {
    let commands = all_commands(spec);
    let (level, reason) = risk_summary(spec);
    let request = inline_approval::ApprovalRequest::from_spec(
        spec,
        provider_route(spec),
        verifier_summary(spec),
        commands,
        expected_effects(spec),
        estimated_cost(spec),
        (level, reason),
    );
    inline_approval::render_card(&request, spec_path)
}

fn print_diff_preview(spec_path: &Path, spec: &AgentSpec) {
    println!("Effect preview");
    println!("  no generated patch exists before execution");
    println!("  draft: {}", spec_path.display());
    println!("  expected effects: {}", expected_effects(spec));
    println!("  allowed paths: {}", list_or_none(&spec.scope.allow));
    println!("  denied paths: {}", list_or_none(&spec.scope.deny));
    print!(
        "{}",
        inline_approval::render_diff_preview("# planned scope\n+ allowed paths\n- denied paths\n")
    );
    println!("  run `/diff` after execution to inspect actual changes");
}

fn print_verify_details(spec: &AgentSpec) {
    println!("Verify details");
    println!(
        "  profile: {}",
        spec.verify.profile.as_deref().unwrap_or("default")
    );
    if spec.verify.commands.is_empty() {
        println!("  commands: <none>");
    } else {
        for command in &spec.verify.commands {
            println!("  command: {command}");
        }
    }
    if let Some(runtime) = &spec.verify.runtime {
        println!(
            "  runtime: {} timeout {}s",
            runtime.base_url, runtime.timeout_secs
        );
        for route in &spec.verify.routes {
            println!("  route: {} -> {}", route.path, route.expect);
        }
    }
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

fn provider_route(spec: &AgentSpec) -> String {
    let default = spec.agent.adapter.as_deref().unwrap_or("command");
    let roles = [
        ("planner", spec.agents.planner.as_ref()),
        ("executor", spec.agents.executor.as_ref()),
        ("reviewer", spec.agents.reviewer.as_ref()),
        ("repair", spec.agents.repair.as_ref()),
    ]
    .into_iter()
    .filter_map(|(role, agent)| {
        agent
            .and_then(|config| config.adapter.as_deref())
            .map(|adapter| format!("{role}={adapter}"))
    })
    .collect::<Vec<_>>();
    if roles.is_empty() {
        default.to_string()
    } else {
        roles.join(" ")
    }
}

fn verifier_summary(spec: &AgentSpec) -> String {
    let mut parts = Vec::new();
    parts.push(
        spec.verify
            .profile
            .as_deref()
            .unwrap_or("default")
            .to_string(),
    );
    parts.extend(spec.verify.commands.iter().cloned());
    if let Some(runtime) = &spec.verify.runtime {
        parts.push(format!(
            "runtime {} routes:{}",
            runtime.base_url,
            spec.verify.routes.len()
        ));
    }
    parts.join(", ")
}

fn expected_effects(spec: &AgentSpec) -> String {
    let mut effects = Vec::new();
    if !spec.scope.allow.is_empty() {
        effects.push("file edits");
    }
    if all_commands(spec)
        .iter()
        .any(|command| command.contains("http"))
    {
        effects.push("network");
    }
    if spec.transaction.memory_promotion != "none" {
        effects.push("memory promotion");
    }
    if spec.transaction.commit_on_success {
        effects.push("git commit");
    }
    if effects.is_empty() {
        "plan only".to_string()
    } else {
        effects.join(", ")
    }
}

fn estimated_cost(spec: &AgentSpec) -> String {
    spec.topology
        .routing
        .max_estimated_cost_usd
        .map(|value| format!("<= ${value:.4}"))
        .unwrap_or_else(|| "unknown".to_string())
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

#[cfg(test)]
mod tests {
    use crate::spec::*;

    use super::*;

    #[test]
    fn renders_card_based_plan_summary() {
        let spec = AgentSpec {
            task: TaskSpec {
                id: "add_page".to_string(),
                kind: "code.add_page".to_string(),
                title: Some("Add courses page".to_string()),
                target: Some("/courses".to_string()),
            },
            agent: AgentConfig {
                adapter: Some("codex".to_string()),
                ..AgentConfig::default()
            },
            agents: RoleAgents::default(),
            topology: TopologySpec::default(),
            workspace: WorkspaceSpec {
                kind: "code.git".to_string(),
                isolation: Some("git_worktree".to_string()),
                root: None,
            },
            skills: Vec::new(),
            execution: ExecutionSpec {
                commands: vec!["npm run build".to_string()],
                sandbox: SandboxSpec::default(),
            },
            scope: ScopeSpec {
                allow: vec!["src/**".to_string()],
                deny: vec![".env".to_string()],
            },
            rules: Vec::new(),
            verify: VerifySpec {
                profile: Some("web".to_string()),
                commands: vec!["npm test".to_string()],
                runtime: None,
                routes: Vec::new(),
            },
            review: ReviewSpec::default(),
            repair: RepairSpec::default(),
            transaction: TransactionSpec::default(),
        };

        let output = render_plan_card(&spec, None);

        assert!(output.contains("AgentHub Plan"));
        assert!(output.contains("provider route: codex"));
        assert!(output.contains("target files: src/**"));
        assert!(output.contains("effects: file edits"));
        assert!(output.contains("[Enter] run"));
    }
}
