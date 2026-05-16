use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub binary: Option<&'static str>,
    pub endpoint_env: Option<&'static str>,
    pub template: Option<&'static str>,
    pub credential_env: &'static [&'static str],
    pub credential_paths: &'static [&'static str],
    pub auth_hint: &'static str,
    pub status_hint: &'static str,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub info: ProviderInfo,
    pub available: bool,
    pub path: Option<PathBuf>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub api_key_env: Option<String>,
    pub profile_kind: Option<String>,
    pub is_default: bool,
}

pub fn supported() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "command".to_string(),
            binary: None,
            endpoint_env: None,
            template: None,
            credential_env: &[],
            credential_paths: &[],
            auth_hint: "no authentication required",
            status_hint: "built-in runner is always available",
            note: "built-in deterministic command runner",
        },
        ProviderInfo {
            id: "codex".to_string(),
            binary: Some("codex"),
            endpoint_env: None,
            template: Some("codex exec --sandbox workspace-write - < {prompt}"),
            credential_env: &["OPENAI_API_KEY"],
            credential_paths: &["$CODEX_HOME/auth.json", "$HOME/.codex/auth.json"],
            auth_hint: "Codex CLI manages login; run the Codex CLI directly if live calls fail",
            status_hint: "AgentHub checks binary, version, template, and known credential markers",
            note: "install the Codex CLI and make `codex` available on PATH",
        },
        ProviderInfo {
            id: "gemini".to_string(),
            binary: Some("gemini"),
            endpoint_env: None,
            template: Some("gemini --prompt-file {prompt}"),
            credential_env: &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
            credential_paths: &["$HOME/.gemini"],
            auth_hint:
                "Gemini CLI manages login; configure Gemini CLI credentials before live calls",
            status_hint: "AgentHub checks binary, version, template, and known credential markers",
            note: "install the Gemini CLI and make `gemini` available on PATH",
        },
        ProviderInfo {
            id: "kimi".to_string(),
            binary: Some("kimi"),
            endpoint_env: None,
            template: Some("kimi --print --afk --input-format text < {prompt}"),
            credential_env: &["KIMI_API_KEY", "MOONSHOT_API_KEY"],
            credential_paths: &["$HOME/.kimi", "$HOME/.config/kimi"],
            auth_hint: "Kimi CLI manages login; configure Kimi credentials before live calls",
            status_hint: "AgentHub checks binary, version, template, and known credential markers",
            note: "install the Kimi CLI and make `kimi` available on PATH",
        },
        ProviderInfo {
            id: "openai-http".to_string(),
            binary: None,
            endpoint_env: Some("AGENTHUB_OPENAI_COMPAT_BASE_URL"),
            template: None,
            credential_env: &["AGENTHUB_OPENAI_COMPAT_API_KEY"],
            credential_paths: &[],
            auth_hint: "set AGENTHUB_OPENAI_COMPAT_API_KEY when the endpoint requires bearer auth",
            status_hint:
                "providers test performs a live completion request and optional model probe",
            note:
                "set AGENTHUB_OPENAI_COMPAT_BASE_URL for an OpenAI-compatible HTTP/HTTPS endpoint",
        },
    ]
}
