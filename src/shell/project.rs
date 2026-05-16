use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub(super) fn resolve(current: &Path, input: &str) -> Result<PathBuf> {
    let target = input.trim();
    if target.is_empty() {
        return Err(anyhow!("usage: /cd <folder>"));
    }
    let expanded = expand_home(target);
    let path = PathBuf::from(expanded);
    let absolute = if path.is_absolute() {
        path
    } else {
        current.join(path)
    };
    Ok(absolute
        .canonicalize()
        .unwrap_or_else(|_| normalize(absolute)))
}

fn expand_home(input: &str) -> String {
    if input == "~" {
        return home_dir().unwrap_or_else(|| input.to_string());
    }
    input
        .strip_prefix("~/")
        .and_then(|rest| home_dir().map(|home| format!("{home}/{rest}")))
        .unwrap_or_else(|| input.to_string())
}

fn home_dir() -> Option<String> {
    std::env::var("HOME").ok().filter(|value| !value.is_empty())
}

#[cfg(windows)]
fn normalize(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(stripped) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{stripped}"));
    }
    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped);
    }
    path
}

#[cfg(not(windows))]
fn normalize(path: PathBuf) -> PathBuf {
    path
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[test]
    fn resolves_relative_project_path() {
        let root = tempfile::tempdir().unwrap();
        let child = root.path().join("other");
        assert_eq!(resolve(root.path(), "other").unwrap(), child);
    }

    #[test]
    fn rejects_empty_target() {
        let root = tempfile::tempdir().unwrap();
        assert!(resolve(root.path(), " ").is_err());
    }
}
