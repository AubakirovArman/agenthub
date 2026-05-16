use crate::llm_gateway::types::ToolCall;
use crate::tool_permissions::{
    classify_shell_command, ToolPermissionDecision, ToolPermissionProfile, ToolRisk,
};

pub fn classify_tool_call(call: &ToolCall) -> ToolPermissionDecision {
    let normalized = call.name.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "shell" | "bash" | "execute_command" | "run_command"
    ) {
        return classify_shell_command(
            call.arguments
                .get("command")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(call.raw_arguments.as_str()),
        );
    }

    let (profile, approval_required, risk, reason) = match normalized.as_str() {
        "read_file" | "list_dir" | "search" | "grep" => (
            ToolPermissionProfile::ReadOnly,
            false,
            ToolRisk::Low,
            "native tool call is read-only",
        ),
        "write_file" | "apply_patch" | "edit_file" => (
            ToolPermissionProfile::WorkspaceWrite,
            false,
            ToolRisk::Medium,
            "native tool call can write workspace files",
        ),
        "agenthub_command_plan" => (
            ToolPermissionProfile::WorkspaceWrite,
            false,
            ToolRisk::Medium,
            "native tool call proposes commands that AgentHub permission-checks before execution",
        ),
        _ => (
            ToolPermissionProfile::WorkspaceWrite,
            true,
            ToolRisk::High,
            "unknown native tool call requires explicit approval before execution",
        ),
    };

    ToolPermissionDecision {
        tool: call.name.clone(),
        action: summarize_action(call),
        profile,
        approval_required,
        risk,
        reason: reason.to_string(),
    }
}

fn summarize_action(call: &ToolCall) -> String {
    if let Some(command) = call
        .arguments
        .get("command")
        .and_then(serde_json::Value::as_str)
    {
        return command.to_string();
    }
    if let Some(path) = call
        .arguments
        .get("path")
        .and_then(serde_json::Value::as_str)
    {
        return path.to_string();
    }
    call.raw_arguments.clone()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn shell_tool_calls_use_shell_permission_classifier() {
        let call = ToolCall {
            id: "call-1".to_string(),
            name: "execute_command".to_string(),
            arguments: json!({ "command": "kubectl delete pod api-1" }),
            raw_arguments: "{\"command\":\"kubectl delete pod api-1\"}".to_string(),
        };

        let decision = classify_tool_call(&call);

        assert_eq!(decision.tool, "shell");
        assert!(decision.approval_required);
        assert_eq!(decision.profile, ToolPermissionProfile::OpsHost);
    }

    #[test]
    fn unknown_tool_calls_are_blocked_by_default() {
        let call = ToolCall {
            id: "call-1".to_string(),
            name: "unknown_tool".to_string(),
            arguments: json!({}),
            raw_arguments: "{}".to_string(),
        };

        let decision = classify_tool_call(&call);

        assert!(decision.approval_required);
        assert_eq!(decision.risk, ToolRisk::High);
    }
}
