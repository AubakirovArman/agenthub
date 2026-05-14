use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMaps {
    pub generated_at: DateTime<Utc>,
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
