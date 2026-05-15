use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::process::process_control_label;
use super::RemoteRunner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitPolicy {
    pub timeout_secs: u64,
    pub cpu_cores: Option<f32>,
    pub memory_mb: Option<u64>,
    pub disk_mb: Option<u64>,
    pub network: String,
    pub filesystem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub wall_time_ms: u128,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerMetadata {
    pub runner_id: String,
    pub kind: String,
    pub trust_level: String,
    pub endpoint: Option<String>,
    pub platform: String,
    pub sandbox_level: u8,
    pub process_control: String,
    pub capabilities: Vec<String>,
    pub resource_limits: ResourceLimitPolicy,
}

pub fn metadata_for(
    sandbox_level: u8,
    remote_runner: Option<&RemoteRunner>,
    timeout: Duration,
) -> RunnerMetadata {
    let limits = resource_limits(sandbox_level, timeout);
    match remote_runner {
        Some(runner) => RunnerMetadata {
            runner_id: runner.id.clone(),
            kind: "remote".to_string(),
            trust_level: trust_for_level(sandbox_level).to_string(),
            endpoint: Some(runner.endpoint.clone()),
            platform: std::env::consts::OS.to_string(),
            sandbox_level,
            process_control: "remote_runner_cancel_or_child_kill".to_string(),
            capabilities: vec![
                "timeout".to_string(),
                "remote_dispatch".to_string(),
                "artifact_return".to_string(),
                "cancel_marker".to_string(),
            ],
            resource_limits: limits,
        },
        None => RunnerMetadata {
            runner_id: "local".to_string(),
            kind: "local".to_string(),
            trust_level: trust_for_level(sandbox_level).to_string(),
            endpoint: None,
            platform: std::env::consts::OS.to_string(),
            sandbox_level,
            process_control: process_control_label().to_string(),
            capabilities: local_capabilities(sandbox_level),
            resource_limits: limits,
        },
    }
}

pub fn usage(duration_ms: u128, exit_code: Option<i32>, timed_out: bool) -> ResourceUsage {
    ResourceUsage {
        wall_time_ms: duration_ms,
        exit_code,
        timed_out,
    }
}

fn resource_limits(sandbox_level: u8, timeout: Duration) -> ResourceLimitPolicy {
    ResourceLimitPolicy {
        timeout_secs: timeout.as_secs(),
        cpu_cores: None,
        memory_mb: None,
        disk_mb: None,
        network: if sandbox_level == 0 {
            "host"
        } else {
            "inherit"
        }
        .to_string(),
        filesystem: if sandbox_level == 0 {
            "workspace"
        } else {
            "sanitized_workspace"
        }
        .to_string(),
    }
}

fn local_capabilities(sandbox_level: u8) -> Vec<String> {
    let mut values = vec![
        "timeout".to_string(),
        "process_tree_kill".to_string(),
        "cancel_marker".to_string(),
        "resource_usage_wall_time".to_string(),
    ];
    if sandbox_level > 0 {
        values.push("sanitized_env".to_string());
    }
    values
}

fn trust_for_level(level: u8) -> &'static str {
    match level {
        0 => "local-dev",
        1 => "local-sandbox",
        2 => "team-runner",
        _ => "enterprise-isolated",
    }
}
