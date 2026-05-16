use std::collections::VecDeque;
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::command_runner::{run_shell, CommandResult};
use crate::llm_gateway::{classify_tool_call, ToolCall, ToolDefinition};
use crate::observability::redact_text;
use crate::tool_permissions::{ToolPermissionDecision, ToolPermissionProfile};

const MAX_READ_BYTES: u64 = 64 * 1024;
const MAX_LIST_ENTRIES: usize = 200;
const MAX_SEARCH_FILES: usize = 500;
const MAX_SEARCH_MATCHES: usize = 80;
const MAX_RESULT_CHARS: usize = 16_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionStatus {
    Ok,
    Error,
    ApprovalRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub call_id: String,
    pub name: String,
    pub status: ToolExecutionStatus,
    pub permission: ToolPermissionDecision,
    pub content: Option<Value>,
    pub error: Option<String>,
}

pub fn builtin_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file".to_string(),
            description:
                "Read a bounded UTF-8 text file from the current AgentHub transaction worktree."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative path to read"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "list_dir".to_string(),
            description:
                "List bounded directory entries inside the current AgentHub transaction worktree."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Workspace-relative directory path to list"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "search".to_string(),
            description:
                "Search bounded UTF-8 workspace files for a literal query and return matching lines."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Literal text query to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional workspace-relative file or directory to search"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "shell".to_string(),
            description:
                "Run a short read-only shell inspection command in the current worktree. Mutating or unsafe commands are returned as approval_required instead of being executed."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Read-only non-interactive shell command"
                    }
                },
                "required": ["command"]
            }),
        },
    ]
}

pub fn execute_tool_call(root: &Path, call: &ToolCall) -> ToolExecutionResult {
    let permission = classify_tool_call(call);
    if permission.approval_required || permission.profile != ToolPermissionProfile::ReadOnly {
        return ToolExecutionResult {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: ToolExecutionStatus::ApprovalRequired,
            permission,
            content: None,
            error: Some("tool call requires approval or is not read-only".to_string()),
        };
    }

    let result = match call.name.trim().to_ascii_lowercase().as_str() {
        "read_file" => read_file(root, call),
        "list_dir" => list_dir(root, call),
        "search" | "grep" => search(root, call),
        "shell" | "bash" | "execute_command" | "run_command" => shell(root, call),
        _ => Err(anyhow!("unknown AgentHub builtin tool `{}`", call.name)),
    };

    match result {
        Ok(content) => ToolExecutionResult {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: ToolExecutionStatus::Ok,
            permission,
            content: Some(content),
            error: None,
        },
        Err(error) => ToolExecutionResult {
            call_id: call.id.clone(),
            name: call.name.clone(),
            status: ToolExecutionStatus::Error,
            permission,
            content: None,
            error: Some(error.to_string()),
        },
    }
}

pub fn result_needs_approval(result: &ToolExecutionResult) -> bool {
    matches!(result.status, ToolExecutionStatus::ApprovalRequired)
        || result.permission.approval_required
}

pub fn results_prompt(round: usize, results: &[ToolExecutionResult]) -> Result<String> {
    let value = serde_json::to_value(results)?;
    let rendered = serde_json::to_string_pretty(&value)?;
    Ok(format!(
        "\n\nAgentHub builtin tool results, round {round} (redacted JSON):\n```json\n{}\n```\nContinue the same turn. Use these results to either call another bounded read-only AgentHub tool or call `agenthub_command_plan` with the final non-interactive command plan. Do not repeat completed tool calls unless the result was an error.\n",
        bound_text(&rendered, MAX_RESULT_CHARS)
    ))
}

fn read_file(root: &Path, call: &ToolCall) -> Result<Value> {
    let path = required_string(call, "path")?;
    let resolved = resolve_existing(root, path)?;
    let meta = fs::metadata(&resolved).with_context(|| format!("stat {}", resolved.display()))?;
    if !meta.is_file() {
        return Err(anyhow!("{} is not a file", path));
    }
    let bytes = fs::read(&resolved).with_context(|| format!("read {}", resolved.display()))?;
    let truncated = bytes.len() as u64 > MAX_READ_BYTES;
    let slice = if truncated {
        &bytes[..MAX_READ_BYTES as usize]
    } else {
        &bytes
    };
    let text =
        std::str::from_utf8(slice).with_context(|| format!("{} is not valid UTF-8 text", path))?;
    Ok(json!({
        "path": relative_display(root, &resolved)?,
        "bytes_read": slice.len(),
        "truncated": truncated,
        "text": bound_text(&redact_text(text)?, MAX_RESULT_CHARS),
    }))
}

fn list_dir(root: &Path, call: &ToolCall) -> Result<Value> {
    let path = required_string(call, "path")?;
    let resolved = resolve_existing(root, path)?;
    if !resolved.is_dir() {
        return Err(anyhow!("{} is not a directory", path));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&resolved).with_context(|| format!("read {}", resolved.display()))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        entries.push(json!({
            "name": entry.file_name().to_string_lossy(),
            "kind": if file_type.is_dir() {
                "dir"
            } else if file_type.is_file() {
                "file"
            } else {
                "other"
            },
        }));
        if entries.len() >= MAX_LIST_ENTRIES {
            break;
        }
    }
    Ok(json!({
        "path": relative_display(root, &resolved)?,
        "truncated": entries.len() >= MAX_LIST_ENTRIES,
        "entries": entries,
    }))
}

fn search(root: &Path, call: &ToolCall) -> Result<Value> {
    let query = required_string(call, "query")?;
    if query.trim().is_empty() {
        return Err(anyhow!("search query is empty"));
    }
    let start = call
        .arguments
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let resolved = resolve_existing(root, start)?;
    let mut matches = Vec::new();
    let mut visited_files = 0usize;
    for file in candidate_files(&resolved)? {
        if visited_files >= MAX_SEARCH_FILES || matches.len() >= MAX_SEARCH_MATCHES {
            break;
        }
        visited_files += 1;
        let Ok(bytes) = fs::read(&file) else {
            continue;
        };
        if bytes.len() as u64 > MAX_READ_BYTES {
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes) else {
            continue;
        };
        for (line_index, line) in text.lines().enumerate() {
            if line.contains(query) {
                matches.push(json!({
                    "path": relative_display(root, &file)?,
                    "line": line_index + 1,
                    "text": bound_text(&redact_text(line)?, 800),
                }));
                if matches.len() >= MAX_SEARCH_MATCHES {
                    break;
                }
            }
        }
    }
    Ok(json!({
        "query": query,
        "path": relative_display(root, &resolved)?,
        "visited_files": visited_files,
        "truncated": visited_files >= MAX_SEARCH_FILES || matches.len() >= MAX_SEARCH_MATCHES,
        "matches": matches,
    }))
}

fn shell(root: &Path, call: &ToolCall) -> Result<Value> {
    let command = required_string(call, "command")?;
    let result = run_shell(command, root, Duration::from_secs(30))?;
    shell_result_json(&result)
}

fn shell_result_json(result: &CommandResult) -> Result<Value> {
    Ok(json!({
        "command": result.command,
        "success": result.success,
        "exit_code": result.exit_code,
        "timed_out": result.timed_out,
        "duration_ms": result.duration_ms,
        "stdout": bound_text(&redact_text(&result.stdout)?, MAX_RESULT_CHARS / 2),
        "stderr": bound_text(&redact_text(&result.stderr)?, MAX_RESULT_CHARS / 2),
    }))
}

fn required_string<'a>(call: &'a ToolCall, key: &str) -> Result<&'a str> {
    call.arguments
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("tool `{}` missing required string `{}`", call.name, key))
}

fn resolve_existing(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.contains('\0') {
        return Err(anyhow!("path contains NUL byte"));
    }
    let raw = Path::new(relative);
    if raw.is_absolute()
        || raw
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(anyhow!("path escapes AgentHub worktree: {}", relative));
    }
    let root = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;
    let candidate = root.join(raw);
    let resolved = candidate
        .canonicalize()
        .with_context(|| format!("canonicalize {}", candidate.display()))?;
    if !resolved.starts_with(&root) {
        return Err(anyhow!("path escapes AgentHub worktree: {}", relative));
    }
    Ok(resolved)
}

fn relative_display(root: &Path, resolved: &Path) -> Result<String> {
    let root = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;
    Ok(resolved
        .strip_prefix(root)
        .unwrap_or(resolved)
        .display()
        .to_string())
}

fn candidate_files(start: &Path) -> Result<Vec<PathBuf>> {
    if start.is_file() {
        return Ok(vec![start.to_path_buf()]);
    }
    let mut files = Vec::new();
    let mut queue = VecDeque::from([start.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if should_skip(&name) {
                continue;
            }
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                queue.push_back(path);
            } else if file_type.is_file() {
                files.push(path);
            }
            if files.len() >= MAX_SEARCH_FILES {
                return Ok(files);
            }
        }
    }
    Ok(files)
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".agent" | "target" | "node_modules" | ".venv" | "__pycache__"
    )
}

fn bound_text(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out = text
        .chars()
        .take(limit.saturating_sub(32))
        .collect::<String>();
    out.push_str("\n... truncated by AgentHub ...");
    out
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn read_file_blocks_path_escape() -> Result<()> {
        let dir = TempDir::new()?;
        let outside = dir.path().join("../outside.txt");
        let call = ToolCall {
            id: "call-1".to_string(),
            name: "read_file".to_string(),
            arguments: json!({ "path": outside.display().to_string() }),
            raw_arguments: "{}".to_string(),
        };

        let result = execute_tool_call(dir.path(), &call);

        assert!(matches!(result.status, ToolExecutionStatus::Error));
        assert!(result.error.unwrap().contains("escapes AgentHub worktree"));
        Ok(())
    }

    #[test]
    fn read_and_search_workspace_text() -> Result<()> {
        let dir = TempDir::new()?;
        fs::write(dir.path().join("README.md"), "hello AgentHub\nsecond\n")?;
        let read = ToolCall {
            id: "call-read".to_string(),
            name: "read_file".to_string(),
            arguments: json!({ "path": "README.md" }),
            raw_arguments: "{}".to_string(),
        };
        let search = ToolCall {
            id: "call-search".to_string(),
            name: "search".to_string(),
            arguments: json!({ "query": "AgentHub", "path": "." }),
            raw_arguments: "{}".to_string(),
        };

        let read_result = execute_tool_call(dir.path(), &read);
        let search_result = execute_tool_call(dir.path(), &search);

        assert!(matches!(read_result.status, ToolExecutionStatus::Ok));
        assert!(serde_json::to_string(&read_result.content)?.contains("hello AgentHub"));
        assert!(matches!(search_result.status, ToolExecutionStatus::Ok));
        assert!(serde_json::to_string(&search_result.content)?.contains("README.md"));
        Ok(())
    }

    #[test]
    fn shell_tool_runs_only_read_only_commands() {
        let dir = TempDir::new().expect("temp dir");
        let read_only = ToolCall {
            id: "call-shell".to_string(),
            name: "shell".to_string(),
            arguments: json!({ "command": "pwd" }),
            raw_arguments: "{}".to_string(),
        };
        let write = ToolCall {
            id: "call-write".to_string(),
            name: "shell".to_string(),
            arguments: json!({ "command": "touch x.txt" }),
            raw_arguments: "{}".to_string(),
        };

        let read_result = execute_tool_call(dir.path(), &read_only);
        let write_result = execute_tool_call(dir.path(), &write);

        assert!(matches!(read_result.status, ToolExecutionStatus::Ok));
        assert!(matches!(
            write_result.status,
            ToolExecutionStatus::ApprovalRequired
        ));
    }
}
