use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub binary: Option<&'static str>,
    pub endpoint_env: Option<&'static str>,
    pub template: Option<&'static str>,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub info: ProviderInfo,
    pub available: bool,
    pub path: Option<PathBuf>,
    pub endpoint: Option<String>,
    pub is_default: bool,
}

pub fn supported() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "command",
            binary: None,
            endpoint_env: None,
            template: None,
            note: "built-in deterministic command runner",
        },
        ProviderInfo {
            id: "codex",
            binary: Some("codex"),
            endpoint_env: None,
            template: Some("codex exec --prompt-file {prompt}"),
            note: "install the Codex CLI and make `codex` available on PATH",
        },
        ProviderInfo {
            id: "gemini",
            binary: Some("gemini"),
            endpoint_env: None,
            template: Some("gemini --prompt-file {prompt}"),
            note: "install the Gemini CLI and make `gemini` available on PATH",
        },
        ProviderInfo {
            id: "kimi",
            binary: Some("kimi"),
            endpoint_env: None,
            template: Some("kimi --prompt-file {prompt}"),
            note: "install the Kimi CLI and make `kimi` available on PATH",
        },
        ProviderInfo {
            id: "openai-http",
            binary: None,
            endpoint_env: Some("AGENTHUB_OPENAI_COMPAT_BASE_URL"),
            template: None,
            note: "set AGENTHUB_OPENAI_COMPAT_BASE_URL for an OpenAI-compatible HTTP endpoint",
        },
    ]
}
