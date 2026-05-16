use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::agent_dir::{self, AgentPaths};
use crate::ui::event_bus::read_tx_events;
use crate::ui::model::{stage_for_journal_state, status_badge, ui_state_for_journal_state};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUiState {
    pub project: String,
    pub generated_at: DateTime<Utc>,
    pub transactions: Vec<UiTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTransaction {
    pub id: String,
    pub status: String,
    pub ui_state: String,
    pub stage: String,
    pub badge: String,
    pub report_path: String,
    pub latest_event: Option<String>,
    pub latest_ts: Option<DateTime<Utc>>,
    pub artifacts: TransactionArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionArtifacts {
    pub report: bool,
    pub diff_guard: bool,
    pub effects: bool,
    pub logs: bool,
}

pub fn collect_project_state(root: &Path) -> Result<ProjectUiState> {
    let paths = AgentPaths::new(root);
    let mut transactions = Vec::new();
    for row in agent_dir::list_transactions(root)?.into_iter().rev() {
        let tx_id = row.id;
        let status = row.status;
        let report_path = row.report_path;
        let events = read_tx_events(root, &tx_id)?;
        let latest = events.last().cloned();
        let stage = stage_for_journal_state(&status);
        let ui_state = ui_state_for_journal_state(&status);
        let tx_dir = paths.tx_dir(&tx_id);
        transactions.push(UiTransaction {
            id: tx_id,
            status: status.clone(),
            ui_state: ui_state.as_str().to_string(),
            stage: latest
                .as_ref()
                .map(|event| event.stage.clone())
                .unwrap_or_else(|| stage.as_str().to_string()),
            badge: status_badge(&status).to_string(),
            report_path: report_path.display().to_string(),
            latest_event: latest.as_ref().map(|event| event.message.clone()),
            latest_ts: latest.as_ref().map(|event| event.ts),
            artifacts: TransactionArtifacts {
                report: report_path.exists(),
                diff_guard: tx_dir.join("diff_guard.json").exists(),
                effects: tx_dir.join("effects.jsonl").exists(),
                logs: tx_dir.join("logs").is_dir(),
            },
        });
    }
    Ok(ProjectUiState {
        project: root.display().to_string(),
        generated_at: Utc::now(),
        transactions,
    })
}
