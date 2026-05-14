use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::command_runner::CommandResult;
use crate::spec::AgentConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRoute {
    pub requested_adapter: String,
    pub selected_adapter: String,
    pub role: String,
    pub model: Option<String>,
    pub fallback_reason: Option<String>,
}

pub fn route(config: &AgentConfig) -> Result<AgentRoute> {
    let requested = config
        .adapter
        .clone()
        .unwrap_or_else(|| "command".to_string());
    let role = config
        .role
        .clone()
        .unwrap_or_else(|| "executor".to_string());
    let model = config.model.clone();

    match requested.as_str() {
        "command" => Ok(AgentRoute {
            requested_adapter: requested.clone(),
            selected_adapter: requested,
            role,
            model,
            fallback_reason: None,
        }),
        "codex" | "kimi" | "gemini" => {
            if executable_available(&requested) {
                Ok(AgentRoute {
                    requested_adapter: requested.clone(),
                    selected_adapter: requested,
                    role,
                    model,
                    fallback_reason: None,
                })
            } else {
                Ok(AgentRoute {
                    requested_adapter: requested.clone(),
                    selected_adapter: "command".to_string(),
                    role,
                    model,
                    fallback_reason: Some(format!(
                        "adapter executable `{requested}` was not found on PATH"
                    )),
                })
            }
        }
        other => Err(anyhow!("unknown agent adapter: {other}")),
    }
}

pub fn supported_adapters() -> Vec<&'static str> {
    vec!["command", "codex", "kimi", "gemini"]
}

pub fn write_agent_trace(tx_dir: &Path, route: &AgentRoute) -> Result<()> {
    write_json(
        tx_dir.join("agent_trace.json").as_path(),
        &json!({
            "route": route,
            "created_at": Utc::now(),
        }),
    )
}

pub fn write_transcript(
    tx_dir: &Path,
    route: &AgentRoute,
    results: &[CommandResult],
) -> Result<()> {
    let path = tx_dir.join("agent_transcript.jsonl");
    for result in results {
        append_jsonl(
            &path,
            &json!({
                "ts": Utc::now(),
                "adapter": route.selected_adapter,
                "role": route.role,
                "command": result.command,
                "exit_code": result.exit_code,
                "success": result.success,
                "timed_out": result.timed_out,
                "duration_ms": result.duration_ms,
            }),
        )?;
    }
    Ok(())
}

fn executable_available(name: &str) -> bool {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .any(|dir| dir.join(name).is_file())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_command_adapter() -> Result<()> {
        let route = route(&AgentConfig::default())?;
        assert_eq!(route.selected_adapter, "command");
        assert_eq!(route.role, "executor");
        Ok(())
    }
}
