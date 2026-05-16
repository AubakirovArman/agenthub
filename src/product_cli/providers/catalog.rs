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
    pub api_key_file: Option<PathBuf>,
    pub profile_kind: Option<String>,
    pub is_default: bool,
}

pub fn supported() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "deepseek".to_string(),
            binary: None,
            endpoint_env: Some("DEEPSEEK_API_BASE_URL"),
            template: None,
            credential_env: &["DEEPSEEK_API_KEY", "ANTHROPIC_AUTH_TOKEN"],
            credential_paths: &[".deepseek"],
            auth_hint: "set DEEPSEEK_API_KEY or place the key in a .deepseek file in the project tree",
            status_hint: "providers test performs a live DeepSeek OpenAI-compatible completion request",
            note: "DeepSeek OpenAI-compatible API endpoint, defaulting to https://api.deepseek.com/v1",
        },
        ProviderInfo {
            id: "kimi".to_string(),
            binary: None,
            endpoint_env: Some("KIMI_API_BASE_URL"),
            template: None,
            credential_env: &["KIMI_API_KEY", "MOONSHOT_API_KEY"],
            credential_paths: &[".kimi"],
            auth_hint: "set KIMI_API_KEY, MOONSHOT_API_KEY, or place the key in a .kimi file in the project tree",
            status_hint:
                "providers test performs a live Kimi OpenAI-compatible completion request",
            note: "Kimi API endpoint, defaulting to https://api.moonshot.ai/v1",
        },
    ]
}
