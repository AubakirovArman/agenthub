use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMaps {
    pub generated_at: DateTime<Utc>,
    pub routes: Vec<RouteMapEntry>,
    pub components: Vec<ComponentMapEntry>,
    pub exports: Vec<ExportMapEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceMapEntries {
    pub routes: Vec<RouteMapEntry>,
    pub components: Vec<ComponentMapEntry>,
    pub exports: Vec<ExportMapEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMapEntry {
    pub route: String,
    pub file: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMapEntry {
    pub name: String,
    pub file: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMapEntry {
    pub symbol: String,
    pub file: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapContextSelection {
    pub routes: Vec<RouteMapEntry>,
    pub components: Vec<ComponentMapEntry>,
    pub exports: Vec<ExportMapEntry>,
    pub validation: MapValidation,
    pub policy: MapContextPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapContextPolicy {
    pub map_based: bool,
    pub full_files_included: bool,
    pub selector: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapValidation {
    pub stale: bool,
    pub missing_maps: Vec<String>,
    pub stale_entries: Vec<StaleMapEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleMapEntry {
    pub map: String,
    pub key: String,
    pub file: String,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
}
