use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_adapter::api_tool_loop::{
    api_execution_plan_tool, build_api_tool_loop_receipt, classify_plan_commands,
    parse_api_execution_plan_from_response, write_api_tool_loop_receipt, ApiExecutionPlan,
};
use crate::agent_adapter::transcript::write_adapter_run;
use crate::agent_adapter::AgentRoute;
use crate::command_runner::{run_shell_with_sandbox_logged, CommandSandbox, RemoteRunner};
use crate::llm_gateway::{complete_with_retry, HttpProvider, LlmRequest, RetryPolicy, ToolChoice};
use crate::observability::{redact_text, redact_value};
use crate::product_cli::providers;
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
    if route.uses_api_provider() {
        let run = invoke_api_provider(spec, tx_dir, worktree, route, remote_runner, prompt_path)?;
        write_invocation(tx_dir, &run)?;
        write_adapter_run(tx_dir, route, &run)?;
        if !run.success {
            return Err(anyhow!(
                "agent adapter `{}` failed for role `{}`",
                run.adapter,
                run.role
            ));
        }
        return Ok(Some(run));
    }
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

fn invoke_api_provider(
    spec: &AgentSpec,
    tx_dir: &Path,
    worktree: &Path,
    route: &AgentRoute,
    remote_runner: Option<&RemoteRunner>,
    prompt_path: PathBuf,
) -> Result<AdapterRun> {
    if route.dry_run {
        return Ok(AdapterRun {
            adapter: route.selected_adapter.clone(),
            role: route.role.clone(),
            command: Some(format!("api://{}", route.selected_adapter)),
            prompt_path,
            success: true,
            exit_code: Some(0),
            stdout: "api adapter dry-run enabled; provider was not called".to_string(),
            stderr: String::new(),
            stdout_path: None,
            stderr_path: None,
            stdout_truncated: false,
            stderr_truncated: false,
            duration_ms: 0,
            dry_run: true,
            remote: false,
            runner: None,
        });
    }

    let started = Instant::now();
    let prompt = fs::read_to_string(&prompt_path)
        .with_context(|| format!("read {}", prompt_path.display()))?;
    let prompt_tokens = estimate_tokens(&prompt);
    let status = provider_status(worktree, route)?;
    let endpoint = status
        .endpoint
        .clone()
        .ok_or_else(|| anyhow!("provider `{}` has no API endpoint", route.selected_adapter))?;
    let model = route.model.clone().or_else(|| status.model.clone());
    let provider = HttpProvider::new(
        endpoint,
        providers::api_key_for_status(&status),
        model.clone(),
    );
    let request = LlmRequest {
        id: format!(
            "api-executor-{}-{}",
            route.role,
            Utc::now().timestamp_millis()
        ),
        role: route.role.clone(),
        provider: route.selected_adapter.clone(),
        model,
        prompt: Some(prompt),
        context_pack_hash: "transaction".to_string(),
        prompt_hash: route.role.clone(),
        prompt_tokens,
        response_format: None,
        tools: vec![api_execution_plan_tool()],
        tool_choice: Some(ToolChoice::Auto),
    };
    let response = complete_with_retry(
        &provider,
        request,
        &RetryPolicy {
            max_attempts: 2,
            backoff_ms: vec![500],
        },
        None,
    )?;
    let (plan, plan_source, content) = parse_api_execution_plan_from_response(&response)?;
    let command_permissions = classify_plan_commands(&plan);
    let tool_loop =
        build_api_tool_loop_receipt(route, &response, &plan_source, &command_permissions);
    write_api_tool_loop_receipt(tx_dir, route, &tool_loop)?;
    if tool_loop.blocked {
        return Err(anyhow!(
            "{}",
            tool_loop
                .blocked_reason
                .as_deref()
                .unwrap_or("API tool loop command blocked")
        ));
    }
    let command_results = run_api_commands(spec, tx_dir, worktree, route, remote_runner, &plan)?;
    let success = command_results.iter().all(|result| result.success);
    let exit_code = if success {
        Some(0)
    } else {
        command_results
            .iter()
            .find(|result| !result.success)
            .and_then(|result| result.exit_code)
    };
    write_api_execution_record(tx_dir, route, &response, &plan, &command_results)?;
    Ok(AdapterRun {
        adapter: route.selected_adapter.clone(),
        role: route.role.clone(),
        command: Some(format!("api://{}", route.selected_adapter)),
        prompt_path,
        success,
        exit_code,
        stdout: redact_text(&content)?,
        stderr: if success {
            String::new()
        } else {
            command_results
                .iter()
                .find(|result| !result.success)
                .map(|result| result.stderr_tail.clone())
                .unwrap_or_else(|| "API executor command failed".to_string())
        },
        stdout_path: None,
        stderr_path: None,
        stdout_truncated: false,
        stderr_truncated: false,
        duration_ms: started.elapsed().as_millis(),
        dry_run: false,
        remote: command_results.iter().any(|result| result.remote),
        runner: command_results
            .iter()
            .find_map(|result| result.runner.clone()),
    })
}

fn provider_status(worktree: &Path, route: &AgentRoute) -> Result<providers::ProviderStatus> {
    providers::statuses(worktree)?
        .into_iter()
        .find(|status| status.info.id == route.selected_adapter)
        .ok_or_else(|| anyhow!("unknown API provider `{}`", route.selected_adapter))
        .and_then(|status| {
            if status.available {
                Ok(status)
            } else {
                Err(anyhow!(
                    "API provider `{}` is not configured; {}",
                    status.info.id,
                    status.info.auth_hint
                ))
            }
        })
}

fn run_api_commands(
    spec: &AgentSpec,
    tx_dir: &Path,
    worktree: &Path,
    route: &AgentRoute,
    remote_runner: Option<&RemoteRunner>,
    plan: &ApiExecutionPlan,
) -> Result<Vec<crate::command_runner::CommandResult>> {
    let mut results = Vec::new();
    for (index, command) in plan.commands.iter().enumerate() {
        if let Some(cancel) = crate::command_runner::read_cancel_request(tx_dir)? {
            crate::command_runner::write_cancel_status(
                tx_dir,
                &crate::command_runner::CancelStatus {
                    cancelled: true,
                    reason: Some(cancel.reason.clone()),
                },
            )?;
            return Err(anyhow!("transaction cancelled: {}", cancel.reason));
        }
        let result = run_shell_with_sandbox_logged(
            command,
            worktree,
            Duration::from_secs(900),
            adapter_sandbox(spec.execution.sandbox.level, remote_runner),
            &tx_dir.join("logs"),
            &format!("api-{}-{index}", route.role),
        )?;
        let success = result.success;
        results.push(result);
        if !success {
            break;
        }
    }
    Ok(results)
}

fn write_api_execution_record(
    tx_dir: &Path,
    route: &AgentRoute,
    response: &crate::llm_gateway::LlmResponse,
    plan: &ApiExecutionPlan,
    command_results: &[crate::command_runner::CommandResult],
) -> Result<()> {
    let path = tx_dir.join(format!("api_execution_{}.json", route.role));
    let record = redact_value(&json!({
        "created_at": Utc::now(),
        "adapter": route.selected_adapter,
        "role": route.role,
        "response": response,
        "plan": plan,
        "command_results": command_results,
    }))?;
    fs::write(&path, serde_json::to_string_pretty(&record)?)
        .with_context(|| format!("write {}", path.display()))
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
        "# AgentHub Adapter Prompt\n\nRole: {}\nAdapter: {}\nModel: {}\nTask: {} ({})\nTitle: {}\nTarget: {}\nWorkspace: {}\n\nInstructions:\n- Edit files directly in the current AgentHub worktree.\n- Stay inside the allowed scope and avoid denied paths.\n- Keep changes focused on this task.\n- If you are an API provider with tools, call `agenthub_command_plan` with {{\"summary\":\"short summary\",\"commands\":[\"shell command 1\",\"shell command 2\"]}}.\n- If tools are unavailable, return only that JSON object as plain content.\n- API commands must be non-interactive, deterministic, and must create or edit files using shell-safe commands such as mkdir, printf, cat <<'EOF', npm, cargo, or test runners.\n- Do not include sudo, reboot, destructive disk commands, or commands that touch denied paths.\n\nSkills:\n{}\n\nRules:\n{}\n\nAllowed paths:\n{}\n\nDenied paths:\n{}\n\nExecution commands after adapter step:\n{}\n\nReview commands:\n{}\n\nRepair commands:\n{}\n\nVerifier commands:\n{}\n",
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

fn estimate_tokens(value: &str) -> usize {
    (value.len() / 4).max(1)
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
