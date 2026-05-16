use std::path::Path;

use anyhow::{anyhow, Result};

use crate::product_cli::config;

pub fn set_role_provider(project_root: &Path, role: &str, provider: &str) -> Result<String> {
    validate_role(role)?;
    let status = super::status_for(project_root, provider)?;
    config::set_value(
        project_root,
        &format!("provider.role.{role}"),
        &status.info.id,
    )?;
    Ok(format!(
        "role\t{}\t{}\tavailable:{}\n",
        role, status.info.id, status.available
    ))
}

pub fn set_role_fallback(project_root: &Path, role: &str, providers: &[String]) -> Result<String> {
    validate_role(role)?;
    if providers.is_empty() {
        return Err(anyhow!("at least one fallback provider is required"));
    }
    let mut ids = Vec::new();
    let mut availability = Vec::new();
    for provider in providers {
        let status = super::status_for(project_root, provider)?;
        ids.push(status.info.id.clone());
        availability.push(format!("{}:{}", status.info.id, status.available));
    }
    config::set_value(
        project_root,
        &format!("provider.fallback.{role}"),
        &ids.join(","),
    )?;
    Ok(format!(
        "fallback\t{}\t{}\t{}\n",
        role,
        ids.join(","),
        availability.join(",")
    ))
}

fn validate_role(role: &str) -> Result<()> {
    let valid = [
        "planner",
        "executor",
        "reviewer",
        "repair",
        "generator",
        "critic",
        "researcher",
        "aggregator",
        "manager",
        "worker",
    ];
    if valid.contains(&role) {
        Ok(())
    } else {
        Err(anyhow!("unknown provider role `{role}`"))
    }
}
