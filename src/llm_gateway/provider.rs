use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;

use crate::command_runner;
use crate::llm_gateway::types::{LlmRequest, LlmResponse, ProviderMetadata, TokenCount};
use crate::observability::{redact_text, write_jsonl};

pub trait LlmProvider {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;
    fn stream_capability(&self) -> ProviderMetadata;
    fn count_tokens(&self, input: &str) -> Result<TokenCount>;
}

#[derive(Debug, Clone)]
pub struct CliProvider {
    adapter: String,
    model: Option<String>,
    command_template: Option<String>,
    workdir: PathBuf,
    transcript_path: Option<PathBuf>,
}

impl CliProvider {
    pub fn new(adapter: impl Into<String>, model: Option<String>) -> Self {
        Self {
            adapter: adapter.into(),
            model,
            command_template: None,
            workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            transcript_path: None,
        }
    }

    pub fn with_command_template(mut self, template: impl Into<String>) -> Self {
        self.command_template = Some(template.into());
        self
    }

    pub fn with_workdir(mut self, workdir: impl Into<PathBuf>) -> Self {
        self.workdir = workdir.into();
        self
    }

    pub fn with_transcript_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.transcript_path = Some(path.into());
        self
    }
}

impl LlmProvider for CliProvider {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        if request.prompt.is_some() && self.command_template.is_some() {
            return self.complete_real(request);
        }
        let status = if self.model.is_some() {
            "planned_cli_wrapper_with_model"
        } else {
            "planned_cli_wrapper"
        };
        Ok(LlmResponse {
            request_id: request.id,
            status: status.to_string(),
            content: None,
            completion_tokens: 0,
            tool_calls: Vec::new(),
            error: None,
        })
    }

    fn stream_capability(&self) -> ProviderMetadata {
        metadata_for_adapter(&self.adapter)
    }

    fn count_tokens(&self, input: &str) -> Result<TokenCount> {
        let prompt_tokens = (input.len() / 4).max(1);
        Ok(TokenCount {
            prompt_tokens,
            completion_tokens: 0,
            total_tokens: prompt_tokens,
            method: "estimated_chars_div_4".to_string(),
        })
    }
}

impl CliProvider {
    fn complete_real(&self, request: LlmRequest) -> Result<LlmResponse> {
        let prompt = request.prompt.clone().unwrap_or_default();
        let prompt_path = write_prompt_file(&self.workdir, &request.id, &prompt)?;
        let command = render_command(
            self.command_template.as_ref().expect("checked by caller"),
            &prompt_path,
            self.model.as_deref().or(request.model.as_deref()),
            &request.role,
        );
        let result = command_runner::run_shell(&command, &self.workdir, Duration::from_secs(900))?;
        let content = if result.success {
            Some(redact_text(&result.stdout)?)
        } else {
            None
        };
        let error = if result.success {
            None
        } else {
            Some(redact_text(&result.stderr)?)
        };
        let response = LlmResponse {
            request_id: request.id.clone(),
            status: if result.success { "ok" } else { "error" }.to_string(),
            content,
            completion_tokens: estimate_tokens(&result.stdout),
            tool_calls: Vec::new(),
            error,
        };
        if let Some(path) = &self.transcript_path {
            write_jsonl(
                path,
                &json!({
                    "ts": Utc::now(),
                    "provider": self.adapter,
                    "request_id": request.id,
                    "command": command,
                    "success": result.success,
                    "exit_code": result.exit_code,
                    "stdout": redact_text(&result.stdout)?,
                    "stderr": redact_text(&result.stderr)?,
                    "duration_ms": result.duration_ms,
                }),
            )?;
        }
        Ok(response)
    }
}

pub fn metadata_for_adapter(adapter: &str) -> ProviderMetadata {
    match adapter {
        "command" => metadata(adapter, "local_command", false, false),
        "deepseek" | "kimi" => metadata(adapter, "api_provider", true, true),
        other => metadata(other, "unknown", false, false),
    }
}

fn metadata(
    adapter: &str,
    kind: &str,
    supports_api: bool,
    supports_streaming: bool,
) -> ProviderMetadata {
    ProviderMetadata {
        id: adapter.to_string(),
        kind: kind.to_string(),
        supports_api,
        supports_streaming,
        token_counting: "estimated".to_string(),
    }
}

fn write_prompt_file(workdir: &Path, request_id: &str, prompt: &str) -> Result<PathBuf> {
    let dir = workdir.join(".agent/llm_gateway");
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let path = dir.join(format!("{request_id}.prompt.txt"));
    fs::write(&path, prompt).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn render_command(template: &str, prompt_path: &Path, model: Option<&str>, role: &str) -> String {
    template
        .replace("{prompt}", &shell_quote(&prompt_path.display().to_string()))
        .replace("{model}", &shell_quote(model.unwrap_or_default()))
        .replace("{role}", &shell_quote(role))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn estimate_tokens(value: &str) -> usize {
    (value.len() / 4).max(1)
}
