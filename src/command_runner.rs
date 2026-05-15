mod cancel;
mod local;
mod metadata;
mod output;
mod process;
mod remote;
mod sandbox;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use cancel::{read_cancel_request, write_cancel_request, write_cancel_status, CancelStatus};
pub use local::{
    run_shell, run_shell_with_sandbox, run_shell_with_sandbox_logged, spawn_shell, SupervisedChild,
};
pub use metadata::{metadata_for, ResourceLimitPolicy, ResourceUsage, RunnerMetadata};
pub use remote::RemoteRunner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub stdout: String,
    pub stderr: String,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub sandbox_level: u8,
    pub remote: bool,
    pub runner: Option<String>,
    pub resource_usage: ResourceUsage,
    pub runner_metadata: RunnerMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct CommandSandbox {
    pub level: u8,
    pub remote_runner: Option<RemoteRunner>,
}

impl CommandSandbox {
    pub fn level(level: u8) -> Self {
        Self {
            level,
            remote_runner: None,
        }
    }
}
