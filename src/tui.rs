mod collect;
mod live;
mod provider_render;
mod providers;
mod read;
mod render;
#[cfg(test)]
mod tests;
mod tool_cards;

use std::path::Path;

use anyhow::Result;

pub use collect::collect_dashboard;
pub use live::{live_dashboard, LiveOptions};
pub use render::render_dashboard;

#[derive(Debug, Clone, Default)]
pub struct Dashboard {
    pub project: String,
    pub summary: DashboardSummary,
    pub shell: ShellPanel,
    pub transactions: Vec<TransactionSummary>,
    pub latest: Option<LatestTransaction>,
    pub providers: ProviderPanel,
    pub memory: MemoryPanel,
    pub approvals: ApprovalPanel,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DashboardSummary {
    pub total: usize,
    pub committed: usize,
    pub rolled_back: usize,
    pub blocked: usize,
    pub running: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ShellPanel {
    pub status: ShellStatusLine,
    pub composer: ComposerPanel,
    pub transcript: Vec<TranscriptLine>,
    pub event_rail: Vec<EventRailItem>,
    pub tool_cards: Vec<ToolCard>,
}

#[derive(Debug, Clone, Default)]
pub struct ShellStatusLine {
    pub mode: String,
    pub provider: String,
    pub provider_ready: bool,
    pub model: Option<String>,
    pub git_state: String,
    pub agent_state: String,
    pub chat_id: Option<String>,
    pub chat_title: Option<String>,
    pub prompt_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
    pub estimated_cost_usd: Option<f64>,
    pub controls: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ComposerPanel {
    pub prompt: String,
    pub slash_palette: Vec<SlashPaletteItem>,
    pub context_mentions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SlashPaletteItem {
    pub command: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct TranscriptLine {
    pub at: String,
    pub speaker: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct EventRailItem {
    pub at: String,
    pub state: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ToolCard {
    pub kind: String,
    pub state: String,
    pub title: String,
    pub detail: String,
    pub link: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransactionSummary {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct LatestTransaction {
    pub id: String,
    pub status: String,
    pub stage: Option<String>,
    pub last_event: Option<String>,
    pub dag_nodes: usize,
    pub dag_edges: usize,
    pub dag_roles: Vec<String>,
    pub verifier_passed: Option<bool>,
    pub verifier_tail: Vec<String>,
    pub cost_usd: Option<f64>,
    pub estimated_tokens: Option<usize>,
    pub effects: usize,
    pub provider: Option<String>,
    pub heartbeat_node: Option<String>,
    pub last_output_sec: Option<u64>,
    pub output_tail: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryPanel {
    pub committed: usize,
    pub failed_attempts: usize,
    pub recent_changes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ApprovalPanel {
    pub specs: Vec<String>,
    pub blocked_transactions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderPanel {
    pub default_provider: String,
    pub ready: usize,
    pub missing: usize,
    pub profiles: usize,
    pub statuses: Vec<ProviderStatusLine>,
    pub roles: Vec<ProviderRoleLine>,
}

#[derive(Debug, Clone)]
pub struct ProviderStatusLine {
    pub id: String,
    pub state: String,
    pub is_default: bool,
    pub detail: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderRoleLine {
    pub role: String,
    pub provider: String,
    pub available: Option<bool>,
    pub fallback: Vec<String>,
}

pub fn dashboard_text(project_root: &Path) -> Result<String> {
    let dashboard = collect_dashboard(project_root)?;
    Ok(render_dashboard(&dashboard))
}
