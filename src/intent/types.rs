use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentOptions {
    pub approval_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPreview {
    pub request: String,
    pub inferred_intent: String,
    pub unknowns: Vec<String>,
    pub questions: Vec<ClarificationQuestion>,
    pub defaults: ResolvedDefaults,
    pub approval_required: bool,
    pub agent_spec_yaml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub id: String,
    pub question: String,
    pub required: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDefaults {
    pub workspace_type: String,
    pub workspace_isolation: String,
    pub agent_adapter: String,
    pub agent_role: String,
    pub verify_profile: String,
    pub max_repair_attempts: u32,
    pub commit_on_success: bool,
    pub memory_promotion: String,
}
