use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::analytics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillScorecard {
    pub id: String,
    pub runs: usize,
    pub success_rate: f64,
    pub rollback_rate: f64,
    pub avg_duration_ms: f64,
    pub avg_cost_usd: f64,
    pub known_failures: usize,
}

pub fn scorecards(project_root: &Path) -> Result<Vec<SkillScorecard>> {
    let skills = super::list_available(project_root)?;
    let history = analytics::read_history(project_root)?;
    let mut cards = skills
        .into_iter()
        .map(|skill| {
            let rows = history
                .iter()
                .filter(|row| row.skills.contains(&skill.skill.id))
                .collect::<Vec<_>>();
            let runs = rows.len();
            let success = rows.iter().filter(|row| row.success).count();
            let rollback = rows.iter().filter(|row| row.rollback).count();
            let duration = rows.iter().map(|row| row.duration_ms).sum::<u64>();
            let cost = rows.iter().map(|row| row.cost_usd).sum::<f64>();
            SkillScorecard {
                id: skill.skill.id,
                runs,
                success_rate: rate(success, runs),
                rollback_rate: rate(rollback, runs),
                avg_duration_ms: average(duration as f64, runs),
                avg_cost_usd: average(cost, runs),
                known_failures: skill.common_errors.len(),
            }
        })
        .collect::<Vec<_>>();
    cards.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(cards)
}

fn rate(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn average(total: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}
