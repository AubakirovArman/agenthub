use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::agent_adapter::transcript::write_adapter_run;
use crate::agent_adapter::AgentRoute;
use crate::command_runner::{run_shell_with_sandbox_logged, CommandSandbox, RemoteRunner};
use crate::observability::redact_text;
use crate::spec::AgentSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterRun {
    pub adapter: String,
    pub role: String,
    pub command: Option<String>,
    pub prompt_path: PathBuf,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub duration_ms: u128,
    pub dry_run: bool,
    pub remote: bool,
    pub runner: Option<String>,
}

pub fn invoke_adapter(
    spec: &AgentSpec,
    tx_dir: &Path,
    worktree: &Path,
    route: &AgentRoute,
    remote_runner: Option<&RemoteRunner>,
) -> Result<Option<AdapterRun>> {
    let prompt_path = write_prompt(spec, tx_dir, route)?;
    if !route.uses_external_cli() {
        return Ok(None);
    }

    let command = render_command(route, &prompt_path)?;
    let run = if route.dry_run {
        AdapterRun {
            adapter: route.selected_adapter.clone(),
            role: route.role.clone(),
            command: Some(command),
            prompt_path,
            success: true,
            exit_code: Some(0),
            stdout: "adapter dry-run enabled; external CLI was not executed".to_string(),
            stderr: String::new(),
            stdout_path: None,
            stderr_path: None,
            stdout_truncated: false,
            stderr_truncated: false,
            duration_ms: 0,
            dry_run: true,
            remote: false,
            runner: None,
        }
    } else {
        let result = run_shell_with_sandbox_logged(
            &command,
            worktree,
            Duration::from_secs(900),
            adapter_sandbox(spec.execution.sandbox.level, remote_runner),
            &tx_dir.join("logs"),
            &format!("adapter-{}", route.role),
        )?;
        AdapterRun {
            adapter: route.selected_adapter.clone(),
            role: route.role.clone(),
            command: Some(command),
            prompt_path,
            success: result.success,
            exit_code: result.exit_code,
            stdout: redact_text(&result.stdout)?,
            stderr: redact_text(&result.stderr)?,
            stdout_path: result.stdout_path.clone(),
            stderr_path: result.stderr_path.clone(),
            stdout_truncated: result.stdout_truncated,
            stderr_truncated: result.stderr_truncated,
            duration_ms: result.duration_ms,
            dry_run: false,
            remote: result.remote,
            runner: result.runner,
        }
    };

    write_invocation(tx_dir, &run)?;
    write_adapter_run(tx_dir, route, &run)?;
    if !run.success {
        return Err(anyhow!(
            "agent adapter `{}` failed for role `{}`",
            run.adapter,
            run.role
        ));
    }
    Ok(Some(run))
}

fn adapter_sandbox(level: u8, remote_runner: Option<&RemoteRunner>) -> CommandSandbox {
    CommandSandbox {
        level,
        remote_runner: remote_runner.cloned(),
    }
}

fn write_prompt(spec: &AgentSpec, tx_dir: &Path, route: &AgentRoute) -> Result<PathBuf> {
    let path = tx_dir.join(format!("agent_prompt_{}.md", route.role));
    let prompt = redact_text(&render_prompt(spec, route))?;
    fs::write(&path, prompt).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn render_prompt(spec: &AgentSpec, route: &AgentRoute) -> String {
    format!(
        "# AgentHub Adapter Prompt\n\nRole: {}\nAdapter: {}\nModel: {}\nTask: {} ({})\nTitle: {}\nTarget: {}\nWorkspace: {}\n\nInstructions:\n- Edit files directly in the current AgentHub worktree.\n- Stay inside the allowed scope and avoid denied paths.\n- Keep changes focused on this task.\n\nSkills:\n{}\n\nRules:\n{}\n\nAllowed paths:\n{}\n\nDenied paths:\n{}\n\nExecution commands:\n{}\n\nReview commands:\n{}\n\nRepair commands:\n{}\n\nVerifier commands:\n{}\n",
        route.role,
        route.selected_adapter,
        route.model.as_deref().unwrap_or("<default>"),
        spec.task.id,
        spec.task.kind,
        spec.task.title.as_deref().unwrap_or("<none>"),
        spec.task.target.as_deref().unwrap_or("<none>"),
        spec.workspace.kind,
        list_values(&spec.skills),
        list_values(&spec.rules),
        list_values(&spec.scope.allow),
        list_values(&spec.scope.deny),
        list_commands(&spec.execution.commands),
        list_commands(&spec.review.commands),
        list_commands(&spec.repair.commands),
        list_commands(&spec.verify.commands)
    )
}

fn list_values(values: &[String]) -> String {
    if values.is_empty() {
        return "- <none>".to_string();
    }
    values
        .iter()
        .map(|value| format!("- {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn list_commands(commands: &[String]) -> String {
    if commands.is_empty() {
        return "- <none>".to_string();
    }
    commands
        .iter()
        .map(|command| format!("- `{command}`"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_command(route: &AgentRoute, prompt_path: &Path) -> Result<String> {
    let template = route.command_template.as_ref().ok_or_else(|| {
        anyhow!(
            "adapter `{}` has no command template",
            route.selected_adapter
        )
    })?;
    Ok(template
        .replace("{prompt}", &shell_quote(&prompt_path.display().to_string()))
        .replace("{role}", &shell_quote(&route.role))
        .replace(
            "{model}",
            &shell_quote(route.model.as_deref().unwrap_or_default()),
        ))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn write_invocation(tx_dir: &Path, run: &AdapterRun) -> Result<()> {
    let path = tx_dir.join(format!("adapter_invocation_{}.json", run.role));
    fs::write(
        &path,
        serde_json::to_string_pretty(&serde_json::json!({
            "created_at": Utc::now(),
            "run": run,
        }))?,
    )
    .with_context(|| format!("write {}", path.display()))
}
