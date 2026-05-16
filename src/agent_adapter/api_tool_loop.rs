use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_adapter::AgentRoute;
use crate::llm_gateway::{classify_tool_call, ToolCall, ToolDefinition};
use crate::observability::redact_value;
use crate::tool_permissions::{classify_shell_command, ToolPermissionDecision};
use crate::tool_registry::{builtin_tool_definitions, result_needs_approval, ToolExecutionResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ApiExecutionPlan {
    pub(super) summary: Option<String>,
    pub(super) commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawApiExecutionPlan {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    commands: Option<Vec<String>>,
    #[serde(default)]
    shell_commands: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ApiToolLoopReceipt {
    created_at: chrono::DateTime<Utc>,
    adapter: String,
    role: String,
    response_request_id: String,
    status: String,
    plan_source: String,
    native_tool_calls: Vec<ToolCall>,
    native_tool_permissions: Vec<ToolPermissionDecision>,
    command_permissions: Vec<ToolPermissionDecision>,
    pub(super) blocked: bool,
    pub(super) blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ApiToolResultRound {
    round: usize,
    response_request_id: String,
    tool_calls: Vec<ToolCall>,
    results: Vec<ToolExecutionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ApiToolResultsReceipt {
    created_at: chrono::DateTime<Utc>,
    adapter: String,
    role: String,
    status: String,
    rounds: Vec<ApiToolResultRound>,
    pub(super) blocked: bool,
    pub(super) blocked_reason: Option<String>,
}

pub(super) fn api_execution_plan_tool() -> ToolDefinition {
    ToolDefinition {
        name: "agenthub_command_plan".to_string(),
        description: "Return the exact non-interactive shell commands AgentHub should execute inside the current transaction worktree.".to_string(),
        parameters: json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "summary": {
                    "type": "string",
                    "description": "Short summary of the planned work"
                },
                "commands": {
                    "type": "array",
                    "description": "Non-interactive shell commands to run in order",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 20
                }
            },
            "required": ["commands"]
        }),
    }
}

pub(super) fn api_project_tools() -> Vec<ToolDefinition> {
    let mut tools = vec![api_execution_plan_tool()];
    tools.extend(builtin_tool_definitions());
    tools
}

pub(super) fn parse_api_execution_plan_from_response(
    response: &crate::llm_gateway::LlmResponse,
) -> Result<(ApiExecutionPlan, String, String)> {
    if let Some(call) = response
        .tool_calls
        .iter()
        .find(|call| call.name == "agenthub_command_plan")
    {
        let plan = parse_api_execution_plan_value(&call.arguments)?;
        let content = serde_json::to_string(&call.arguments)?;
        return Ok((plan, format!("native_tool_call:{}", call.id), content));
    }

    let content = response.content.clone().ok_or_else(|| {
        anyhow!("API executor returned an empty response and no native tool call")
    })?;
    let plan = parse_api_execution_plan(&content)?;
    Ok((plan, "content_json_fallback".to_string(), content))
}

pub(super) fn pending_builtin_tool_calls(
    response: &crate::llm_gateway::LlmResponse,
) -> Vec<ToolCall> {
    response
        .tool_calls
        .iter()
        .filter(|call| call.name != "agenthub_command_plan")
        .cloned()
        .collect()
}

pub(super) fn classify_plan_commands(plan: &ApiExecutionPlan) -> Vec<ToolPermissionDecision> {
    plan.commands
        .iter()
        .map(|command| classify_shell_command(command))
        .collect()
}

pub(super) fn build_api_tool_loop_receipt(
    route: &AgentRoute,
    response: &crate::llm_gateway::LlmResponse,
    plan_source: &str,
    command_permissions: &[ToolPermissionDecision],
) -> ApiToolLoopReceipt {
    let native_tool_permissions = response
        .tool_calls
        .iter()
        .map(classify_tool_call)
        .collect::<Vec<_>>();
    let blocked_command = command_permissions
        .iter()
        .find(|permission| permission.approval_required);
    let blocked_reason = blocked_command.map(|permission| {
        format!(
            "API tool loop blocked `{}`: {}",
            permission.action, permission.reason
        )
    });
    ApiToolLoopReceipt {
        created_at: Utc::now(),
        adapter: route.selected_adapter.clone(),
        role: route.role.clone(),
        response_request_id: response.request_id.clone(),
        status: if blocked_reason.is_some() {
            "blocked".to_string()
        } else {
            "ready".to_string()
        },
        plan_source: plan_source.to_string(),
        native_tool_calls: response.tool_calls.clone(),
        native_tool_permissions,
        command_permissions: command_permissions.to_vec(),
        blocked: blocked_reason.is_some(),
        blocked_reason,
    }
}

pub(super) fn write_api_tool_loop_receipt(
    tx_dir: &Path,
    route: &AgentRoute,
    receipt: &ApiToolLoopReceipt,
) -> Result<()> {
    let path = tx_dir.join(format!("tool_loop_{}.json", route.role));
    let record = redact_value(&serde_json::to_value(receipt)?)?;
    fs::write(&path, serde_json::to_string_pretty(&record)?)
        .with_context(|| format!("write {}", path.display()))
}

pub(super) fn build_api_tool_results_receipt(
    route: &AgentRoute,
    rounds: &[ApiToolResultRound],
) -> ApiToolResultsReceipt {
    let blocked_result = rounds
        .iter()
        .flat_map(|round| round.results.iter())
        .find(|result| result_needs_approval(result));
    let blocked_reason = blocked_result.map(|result| {
        format!(
            "API builtin tool `{}` blocked `{}`: {}",
            result.name, result.permission.action, result.permission.reason
        )
    });
    ApiToolResultsReceipt {
        created_at: Utc::now(),
        adapter: route.selected_adapter.clone(),
        role: route.role.clone(),
        status: if blocked_reason.is_some() {
            "blocked".to_string()
        } else {
            "ready".to_string()
        },
        rounds: rounds.to_vec(),
        blocked: blocked_reason.is_some(),
        blocked_reason,
    }
}

pub(super) fn api_tool_result_round(
    round: usize,
    response: &crate::llm_gateway::LlmResponse,
    tool_calls: Vec<ToolCall>,
    results: Vec<ToolExecutionResult>,
) -> ApiToolResultRound {
    ApiToolResultRound {
        round,
        response_request_id: response.request_id.clone(),
        tool_calls,
        results,
    }
}

pub(super) fn write_api_tool_results_receipt(
    tx_dir: &Path,
    route: &AgentRoute,
    receipt: &ApiToolResultsReceipt,
) -> Result<()> {
    let path = tx_dir.join(format!("tool_results_{}.json", route.role));
    let record = redact_value(&serde_json::to_value(receipt)?)?;
    fs::write(&path, serde_json::to_string_pretty(&record)?)
        .with_context(|| format!("write {}", path.display()))
}

fn parse_api_execution_plan(content: &str) -> Result<ApiExecutionPlan> {
    let json_text = extract_json_object(content)
        .ok_or_else(|| anyhow!("API executor did not return a JSON object"))?;
    let raw: RawApiExecutionPlan =
        serde_json::from_str(json_text).context("parse API executor JSON plan")?;
    normalize_api_execution_plan(raw)
}

fn parse_api_execution_plan_value(value: &Value) -> Result<ApiExecutionPlan> {
    let raw: RawApiExecutionPlan =
        serde_json::from_value(value.clone()).context("parse API executor tool plan")?;
    normalize_api_execution_plan(raw)
}

fn normalize_api_execution_plan(raw: RawApiExecutionPlan) -> Result<ApiExecutionPlan> {
    let commands = raw
        .commands
        .or(raw.shell_commands)
        .unwrap_or_default()
        .into_iter()
        .map(|command| command.trim().to_string())
        .filter(|command| !command.is_empty())
        .collect::<Vec<_>>();
    if commands.is_empty() {
        return Err(anyhow!("API executor JSON plan did not include commands"));
    }
    if commands.len() > 20 {
        return Err(anyhow!(
            "API executor returned too many commands: {} (max 20)",
            commands.len()
        ));
    }
    for command in &commands {
        validate_api_command(command)?;
    }
    Ok(ApiExecutionPlan {
        summary: raw.summary,
        commands,
    })
}

fn extract_json_object(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }
    let fenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|value| value.strip_suffix("```"))
        .map(str::trim);
    if let Some(value) = fenced {
        if value.starts_with('{') && value.ends_with('}') {
            return Some(value);
        }
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start < end).then_some(&trimmed[start..=end])
}

fn validate_api_command(command: &str) -> Result<()> {
    let lower = command.to_ascii_lowercase();
    let denied = [
        "sudo ", "su ", "rm -rf /", "mkfs", "shutdown", "reboot", "dd if=", ":(){",
    ];
    if command.contains('\0') || command.lines().count() > 200 {
        return Err(anyhow!("API executor returned an unsafe shell command"));
    }
    if denied.iter().any(|pattern| lower.contains(pattern)) {
        return Err(anyhow!(
            "API executor returned a denied shell command pattern"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn api_execution_plan_prefers_native_tool_call() -> Result<()> {
        let response = crate::llm_gateway::LlmResponse {
            request_id: "req-1".to_string(),
            status: "ok".to_string(),
            content: Some("{\"commands\":[\"false\"]}".to_string()),
            completion_tokens: 1,
            tool_calls: vec![ToolCall {
                id: "call-1".to_string(),
                name: "agenthub_command_plan".to_string(),
                arguments: json!({
                    "summary": "use native call",
                    "commands": ["printf ok > generated/check.txt"]
                }),
                raw_arguments:
                    "{\"summary\":\"use native call\",\"commands\":[\"printf ok > generated/check.txt\"]}"
                        .to_string(),
            }],
            error: None,
        };

        let (plan, source, content) = parse_api_execution_plan_from_response(&response)?;

        assert_eq!(source, "native_tool_call:call-1");
        assert_eq!(plan.commands, vec!["printf ok > generated/check.txt"]);
        assert!(content.contains("use native call"));
        Ok(())
    }

    #[test]
    fn tool_loop_receipt_blocks_commands_that_need_approval() {
        let route = AgentRoute::api("deepseek".to_string(), "executor".to_string(), None, false);
        let response = crate::llm_gateway::LlmResponse {
            request_id: "req-1".to_string(),
            status: "ok".to_string(),
            content: None,
            completion_tokens: 1,
            tool_calls: Vec::new(),
            error: None,
        };
        let plan = ApiExecutionPlan {
            summary: Some("danger".to_string()),
            commands: vec!["sudo reboot".to_string()],
        };
        let permissions = classify_plan_commands(&plan);

        let receipt =
            build_api_tool_loop_receipt(&route, &response, "content_json_fallback", &permissions);

        assert!(receipt.blocked);
        assert_eq!(receipt.status, "blocked");
        assert!(receipt
            .blocked_reason
            .as_deref()
            .unwrap_or_default()
            .contains("sudo reboot"));
    }
}
