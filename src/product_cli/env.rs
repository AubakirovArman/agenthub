use std::path::PathBuf;

pub fn find_executable(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| find_in_paths(name, std::env::split_paths(&paths)))
}

fn find_in_paths<I>(name: &str, paths: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    for dir in paths {
        for candidate in candidates(&dir, name) {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn candidates(dir: &std::path::Path, name: &str) -> Vec<PathBuf> {
    if cfg!(windows) && std::path::Path::new(name).extension().is_none() {
        return pathexts()
            .into_iter()
            .map(|ext| dir.join(format!("{name}{ext}")))
            .collect();
    }
    vec![dir.join(name)]
}

fn pathexts() -> Vec<String> {
    std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".EXE;.BAT;.CMD".to_string())
        .split(';')
        .filter(|item| !item.is_empty())
        .map(|item| item.to_ascii_lowercase())
        .collect()
}
