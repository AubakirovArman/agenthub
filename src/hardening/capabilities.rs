use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatus {
    pub id: String,
    pub supported: bool,
    pub level: String,
    pub detail: String,
}

pub fn detect_capabilities(project_root: &Path) -> Vec<CapabilityStatus> {
    vec![
        cgroups_v2(),
        container_backend(),
        windows_process_control(),
        network_policy(project_root),
        process_tree_kill(),
    ]
}

fn cgroups_v2() -> CapabilityStatus {
    let supported =
        cfg!(target_os = "linux") && Path::new("/sys/fs/cgroup/cgroup.controllers").exists();
    status(
        "linux.cgroups_v2",
        supported,
        if supported {
            "cgroups v2 controllers detected"
        } else {
            "cgroups v2 not detected or not applicable"
        },
    )
}

fn container_backend() -> CapabilityStatus {
    let docker = find_executable("docker");
    let podman = find_executable("podman");
    let supported = docker.is_some() || podman.is_some();
    let detail = docker
        .or(podman)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "docker/podman not found on PATH".to_string());
    status("container.backend", supported, detail)
}

fn windows_process_control() -> CapabilityStatus {
    status(
        "windows.job_objects",
        cfg!(windows),
        if cfg!(windows) {
            "Windows Job Objects backend can be selected"
        } else {
            "not a Windows host"
        },
    )
}

fn network_policy(project_root: &Path) -> CapabilityStatus {
    let configured = std::env::var("AGENTHUB_NETWORK_POLICY")
        .ok()
        .filter(|v| !v.is_empty());
    let file = project_root.join(".agent/policies/network.yaml");
    let supported = configured.is_some() || file.exists();
    status(
        "network.policy",
        supported,
        configured.unwrap_or_else(|| {
            if file.exists() {
                file.display().to_string()
            } else {
                "network policy not configured".to_string()
            }
        }),
    )
}

fn process_tree_kill() -> CapabilityStatus {
    let supported = cfg!(unix) || cfg!(windows);
    status(
        "process.tree_kill",
        supported,
        if cfg!(windows) {
            "Windows taskkill /T /F termination"
        } else if cfg!(unix) {
            "Unix process group termination"
        } else {
            "child kill fallback; full tree control needs OS backend"
        },
    )
}

fn status(id: &str, supported: bool, detail: impl Into<String>) -> CapabilityStatus {
    CapabilityStatus {
        id: id.to_string(),
        supported,
        level: if supported { "ok" } else { "warn" }.to_string(),
        detail: detail.into(),
    }
}

fn find_executable(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}
