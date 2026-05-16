use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiStage {
    Prepare,
    Context,
    Provider,
    Execute,
    DiffGuard,
    Verify,
    Commit,
    Blocked,
    Canceled,
    Finished,
    Failed,
    Unknown,
}

impl UiStage {
    pub fn as_str(self) -> &'static str {
        match self {
            UiStage::Prepare => "prepare",
            UiStage::Context => "context",
            UiStage::Provider => "provider",
            UiStage::Execute => "execute",
            UiStage::DiffGuard => "diff_guard",
            UiStage::Verify => "verify",
            UiStage::Commit => "commit",
            UiStage::Blocked => "blocked",
            UiStage::Canceled => "canceled",
            UiStage::Finished => "finished",
            UiStage::Failed => "failed",
            UiStage::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiState {
    Pending,
    Running,
    NeedsHuman,
    Succeeded,
    Failed,
    Canceled,
    Unknown,
}

impl UiState {
    pub fn as_str(self) -> &'static str {
        match self {
            UiState::Pending => "pending",
            UiState::Running => "running",
            UiState::NeedsHuman => "needs_human",
            UiState::Succeeded => "succeeded",
            UiState::Failed => "failed",
            UiState::Canceled => "canceled",
            UiState::Unknown => "unknown",
        }
    }
}

pub fn stage_for_journal_state(state: &str) -> UiStage {
    let normalized = state.to_ascii_uppercase();
    match normalized.as_str() {
        "CREATED" | "PREPARING" | "READY" => UiStage::Prepare,
        "CONTEXT" | "CONTEXT_BUILT" | "MEMORY_LOADED" => UiStage::Context,
        "PLANNING" | "ROUTING" | "PROVIDER_SELECTED" | "ADAPTIVE_ORCHESTRATION" => {
            UiStage::Provider
        }
        "EXECUTING" | "RUNNING" | "COMMAND_RUNNING" | "AGENT_RUNNING" => UiStage::Execute,
        "DIFF_GUARD" | "DIFF_CHECK" | "EFFECTS_RECORDED" => UiStage::DiffGuard,
        "VERIFYING" | "VERIFIER_RUNNING" | "REVIEWING" => UiStage::Verify,
        "COMMITTING" | "COMMITTED" | "CLOSED" | "DONE" => UiStage::Commit,
        "BLOCKED_ON_HUMAN" | "APPROVAL_REQUIRED" => UiStage::Blocked,
        "CANCEL_REQUESTED" | "CANCELED" => UiStage::Canceled,
        "ROLLED_BACK" | "FAILED" | "ERROR" => UiStage::Failed,
        _ => UiStage::Unknown,
    }
}

pub fn ui_state_for_journal_state(state: &str) -> UiState {
    let normalized = state.to_ascii_uppercase();
    match normalized.as_str() {
        "CREATED" | "READY" => UiState::Pending,
        "COMMITTED" | "CLOSED" | "DONE" => UiState::Succeeded,
        "BLOCKED_ON_HUMAN" | "APPROVAL_REQUIRED" => UiState::NeedsHuman,
        "CANCEL_REQUESTED" | "CANCELED" => UiState::Canceled,
        "ROLLED_BACK" | "FAILED" | "ERROR" => UiState::Failed,
        value if is_final_state(value) => UiState::Succeeded,
        _ => UiState::Running,
    }
}

pub fn is_final_state(state: &str) -> bool {
    matches!(
        state.to_ascii_uppercase().as_str(),
        "COMMITTED" | "ROLLED_BACK" | "BLOCKED_ON_HUMAN" | "CANCELED" | "CLOSED" | "DONE"
    )
}

pub fn status_badge(state: &str) -> &'static str {
    match ui_state_for_journal_state(state) {
        UiState::Pending => "pending",
        UiState::Running => "running",
        UiState::NeedsHuman => "needs-human",
        UiState::Succeeded => "ok",
        UiState::Failed => "failed",
        UiState::Canceled => "canceled",
        UiState::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_journal_states_to_shared_ui_labels() {
        assert_eq!(stage_for_journal_state("EXECUTING"), UiStage::Execute);
        assert_eq!(
            stage_for_journal_state("BLOCKED_ON_HUMAN"),
            UiStage::Blocked
        );
        assert_eq!(ui_state_for_journal_state("COMMITTED"), UiState::Succeeded);
        assert_eq!(ui_state_for_journal_state("FAILED"), UiState::Failed);
        assert!(is_final_state("CANCELED"));
    }
}
