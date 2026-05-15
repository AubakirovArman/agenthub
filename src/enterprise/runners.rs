use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::enterprise::load_policy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerInventory {
    pub default: String,
    pub remote: Vec<RunnerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerRecord {
    pub id: String,
    pub endpoint: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoute {
    pub model: String,
    pub private: bool,
    pub runner: String,
}

pub fn runner_inventory(project_root: &Path) -> Result<RunnerInventory> {
    let policy = load_policy(project_root)?;
    let runners = policy.enterprise.runners;
    Ok(RunnerInventory {
        default: runners.default,
        remote: runners
            .remote
            .into_iter()
            .map(|runner| RunnerRecord {
                id: runner.id,
                endpoint: runner.endpoint,
                labels: runner.labels,
            })
            .collect(),
    })
}

pub fn route_model(project_root: &Path, model: &str) -> Result<ModelRoute> {
    let policy = load_policy(project_root)?;
    let private = policy
        .enterprise
        .model_routing
        .private_models
        .iter()
        .any(|candidate| candidate == model);
    let runner = if private {
        policy
            .enterprise
            .model_routing
            .private_runner
            .unwrap_or(policy.enterprise.runners.default)
    } else {
        policy.enterprise.runners.default
    };
    Ok(ModelRoute {
        model: model.to_string(),
        private,
        runner,
    })
}
