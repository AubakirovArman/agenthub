use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::analytics::AnalyticsRecord;
use crate::skill_registry::SkillManifest;
use crate::spec::AgentSpec;

use super::{RunState, TransactionStatus};

pub(super) struct HistoryInput<'a> {
    pub project_root: &'a Path,
    pub spec: &'a AgentSpec,
    pub tx_id: &'a str,
    pub tx_dir: &'a Path,
    pub state: &'a RunState,
    pub status: TransactionStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub skills: &'a [SkillManifest],
}

pub(super) fn record_history(input: HistoryInput<'_>) -> Result<()> {
    let duration_ms = (input.finished_at - input.started_at)
        .num_milliseconds()
        .max(0) as u64;
    let cost = input.state.cost_profile.as_ref();
    crate::analytics::record(
        input.project_root,
        &AnalyticsRecord {
            version: "analytics.record.v1".to_string(),
            tx_id: input.tx_id.to_string(),
            task_id: input.spec.task.id.clone(),
            task_type: input.spec.task.kind.clone(),
            status: input.status.as_str().to_string(),
            started_at: input.started_at,
            finished_at: input.finished_at,
            duration_ms,
            success: matches!(input.status, TransactionStatus::Committed),
            rollback: matches!(input.status, TransactionStatus::RolledBack),
            repair: input.tx_dir.join("review_repair.json").exists(),
            human_block: matches!(input.status, TransactionStatus::BlockedOnHuman),
            dangerous_diff: input
                .state
                .diff_guard
                .as_ref()
                .is_some_and(|guard| !guard.passed),
            task_class: input
                .state
                .adaptive
                .as_ref()
                .map(|decision| format!("{:?}", decision.task_class)),
            topology: input
                .state
                .adaptive
                .as_ref()
                .map(|decision| decision.selected_topology.clone()),
            model: input
                .state
                .adaptive
                .as_ref()
                .and_then(|decision| decision.model.clone()),
            verifier_profile: input
                .state
                .verifier
                .as_ref()
                .and_then(|verifier| verifier.profile.clone())
                .or_else(|| input.spec.verify.profile.clone()),
            skills: skill_ids(input.skills),
            cost_usd: cost.map(|item| item.total_usd).unwrap_or_default(),
            estimated_tokens: cost.map(|item| item.estimated_tokens).unwrap_or_default(),
        },
    )?;
    Ok(())
}

fn skill_ids(skills: &[SkillManifest]) -> Vec<String> {
    skills
        .iter()
        .map(|manifest| manifest.skill.id.clone())
        .collect()
}
