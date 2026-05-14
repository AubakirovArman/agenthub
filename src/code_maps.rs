mod scan;
mod select;
mod staleness;
#[cfg(test)]
mod tests;
mod types;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::de::DeserializeOwned;

use crate::agent_dir::AgentPaths;

use scan::*;
pub use types::*;

pub fn build(root: &Path) -> Result<WorkspaceMaps> {
    let files = source_files(root)?;
    let mut routes = Vec::new();
    let mut components = Vec::new();
    let mut exports = Vec::new();

    for file in files {
        let rel = relative(root, &file);
        let hash = file_hash(&file)?;
        if let Some(route) = route_from_path(&rel) {
            routes.push(RouteMapEntry {
                route,
                file: rel.clone(),
                hash: hash.clone(),
            });
        }
        if let Some(name) = component_from_path(&rel) {
            components.push(ComponentMapEntry {
                name,
                file: rel.clone(),
                hash: hash.clone(),
            });
        }
        exports.extend(extract_exports(&file, &rel, &hash)?);
    }

    routes.sort_by(|a, b| a.route.cmp(&b.route));
    components.sort_by(|a, b| a.name.cmp(&b.name));
    exports.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    Ok(WorkspaceMaps {
        generated_at: Utc::now(),
        routes,
        components,
        exports,
    })
}

pub fn write(root: &Path) -> Result<WorkspaceMaps> {
    let maps = build(root)?;
    let paths = AgentPaths::new(root);
    fs::create_dir_all(&paths.maps).with_context(|| format!("create {}", paths.maps.display()))?;
    fs::write(
        paths.maps.join("routes.map.json"),
        serde_json::to_string_pretty(&maps.routes)?,
    )?;
    fs::write(
        paths.maps.join("components.map.json"),
        serde_json::to_string_pretty(&maps.components)?,
    )?;
    fs::write(
        paths.maps.join("exports.map.json"),
        serde_json::to_string_pretty(&maps.exports)?,
    )?;
    Ok(maps)
}

pub fn read_existing(root: &Path) -> Result<serde_json::Value> {
    let entries = read_entries(root)?;
    Ok(serde_json::json!({
        "routes": entries.routes,
        "components": entries.components,
        "exports": entries.exports,
    }))
}

pub fn read_entries(root: &Path) -> Result<WorkspaceMapEntries> {
    let paths = AgentPaths::new(root);
    Ok(WorkspaceMapEntries {
        routes: read_map_file(paths.maps.join("routes.map.json").as_path())?,
        components: read_map_file(paths.maps.join("components.map.json").as_path())?,
        exports: read_map_file(paths.maps.join("exports.map.json").as_path())?,
    })
}

pub fn validate_existing(root: &Path) -> Result<MapValidation> {
    let entries = read_entries(root)?;
    validate_entries(root, &entries)
}

pub fn select_context(root: &Path, spec: &crate::spec::AgentSpec) -> Result<MapContextSelection> {
    let entries = read_entries(root)?;
    let validation = validate_entries(root, &entries)?;
    select::for_spec(spec, entries, validation)
}

fn validate_entries(root: &Path, entries: &WorkspaceMapEntries) -> Result<MapValidation> {
    let mut validation = staleness::validate(root, entries)?;
    validation.missing_maps = missing_map_files(root);
    validation.stale = !validation.missing_maps.is_empty() || !validation.stale_entries.is_empty();
    Ok(validation)
}

fn read_map_file<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&content)?)
}

fn missing_map_files(root: &Path) -> Vec<String> {
    let paths = AgentPaths::new(root);
    [
        ("routes.map.json", paths.maps.join("routes.map.json")),
        (
            "components.map.json",
            paths.maps.join("components.map.json"),
        ),
        ("exports.map.json", paths.maps.join("exports.map.json")),
    ]
    .into_iter()
    .filter_map(|(name, path)| (!path.exists()).then_some(name.to_string()))
    .collect()
}
