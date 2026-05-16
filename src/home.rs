use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub fn base_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("AGENTHUB_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(path);
    }
    if cfg!(test) {
        return std::env::temp_dir().join("agenthub-test-home");
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(xdg).join("agenthub");
    }
    home_dir()
        .map(|home| home.join(".local/share/agenthub"))
        .unwrap_or_else(|| PathBuf::from(".agenthub"))
}

pub fn config_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("AGENTHUB_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(path);
    }
    if cfg!(test) {
        return std::env::temp_dir().join("agenthub-test-config");
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(xdg).join("agenthub");
    }
    home_dir()
        .map(|home| home.join(".config/agenthub"))
        .unwrap_or_else(|| PathBuf::from(".agenthub/config"))
}

pub fn global_chats_dir(project_root: &Path) -> PathBuf {
    base_dir()
        .join("sessions")
        .join(project_scope(project_root))
        .join("chats")
}

pub fn global_history_path(project_root: &Path) -> PathBuf {
    base_dir()
        .join("sessions")
        .join(project_scope(project_root))
        .join("history.txt")
}

pub fn global_shell_commands_dir(project_root: &Path) -> PathBuf {
    base_dir()
        .join("sessions")
        .join(project_scope(project_root))
        .join("commands")
}

pub fn global_drafts_dir(project_root: &Path) -> PathBuf {
    base_dir()
        .join("sessions")
        .join(project_scope(project_root))
        .join("drafts")
}

pub fn global_memory_dir() -> PathBuf {
    base_dir().join("memory")
}

pub fn global_config_path() -> PathBuf {
    config_dir().join("config.yaml")
}

pub fn project_has_runtime(root: &Path) -> bool {
    root.join(".agent/project.yaml").exists()
}

pub fn project_has_shell_state(root: &Path) -> bool {
    root.join(".agent/shell").exists() || project_has_runtime(root)
}

fn project_scope(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut hash = Sha256::new();
    hash.update(canonical.to_string_lossy().as_bytes());
    let digest = hash.finalize();
    let mut suffix = String::new();
    for byte in digest.iter().take(6) {
        suffix.push_str(&format!("{byte:02x}"));
    }
    let name = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("workspace")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("{name}-{suffix}")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
