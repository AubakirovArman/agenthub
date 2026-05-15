mod client;
mod server;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use client::fetch_policy;
pub use server::{serve_policy_server, PolicyServer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyServerConfig {
    pub bind: String,
    pub policy_path: PathBuf,
    pub token: Option<String>,
    pub once: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyServerResult {
    pub bind: String,
    pub policy_path: String,
    pub requests: usize,
}
