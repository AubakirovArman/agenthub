use std::path::PathBuf;

pub fn resolve_cli_project(path: PathBuf) -> PathBuf {
    let resolved = path.canonicalize().unwrap_or_else(|_| absolute_path(path));
    normalize_for_external_tools(resolved)
}

fn absolute_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

#[cfg(windows)]
fn normalize_for_external_tools(path: PathBuf) -> PathBuf {
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
fn normalize_for_external_tools(path: PathBuf) -> PathBuf {
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_project_paths_become_absolute() {
        let resolved = resolve_cli_project(PathBuf::from("."));
        assert!(resolved.is_absolute());
    }
}
