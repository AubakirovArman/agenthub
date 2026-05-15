use std::env;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::enterprise::load_policy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretCheck {
    pub name: String,
    pub provider: String,
    pub allowed: bool,
    pub present: bool,
}

pub fn check_secret(project_root: &Path, name: &str) -> Result<SecretCheck> {
    let policy = load_policy(project_root)?;
    let secrets = policy.enterprise.secrets;
    let allowed = secrets.allowed_prefixes.is_empty()
        || secrets
            .allowed_prefixes
            .iter()
            .any(|prefix| name.starts_with(prefix));
    let present = secrets.provider == "env" && env::var_os(name).is_some();
    Ok(SecretCheck {
        name: name.to_string(),
        provider: secrets.provider,
        allowed,
        present,
    })
}

pub fn check_required_secrets(project_root: &Path) -> Result<Vec<SecretCheck>> {
    let policy = load_policy(project_root)?;
    policy
        .enterprise
        .secrets
        .required
        .iter()
        .map(|name| check_secret(project_root, name))
        .collect()
}
