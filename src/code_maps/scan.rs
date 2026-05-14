use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use super::ExportMapEntry;

pub(super) fn source_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let rel = relative(root, &path);
        if entry.file_type()?.is_dir() {
            if should_skip_dir(&rel) {
                continue;
            }
            visit(root, &path, files)?;
        } else if is_source_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn should_skip_dir(path: &str) -> bool {
    matches!(
        path,
        ".git" | "target" | "node_modules" | ".next" | "dist" | "build"
    ) || path.starts_with(".agent/workspaces")
        || path.starts_with(".agent/cache")
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "rs")
    )
}

pub(super) fn route_from_path(path: &str) -> Option<String> {
    let app_prefix = path
        .strip_prefix("src/app/")
        .or_else(|| path.strip_prefix("app/"))?;
    let route_path = app_prefix
        .strip_suffix("/page.tsx")
        .or_else(|| app_prefix.strip_suffix("/page.ts"))
        .or_else(|| app_prefix.strip_suffix("/page.jsx"))
        .or_else(|| app_prefix.strip_suffix("/page.js"))?;
    if route_path.is_empty() {
        Some("/".to_string())
    } else {
        Some(format!("/{}", route_path))
    }
}

pub(super) fn component_from_path(path: &str) -> Option<String> {
    if !(path.starts_with("src/components/") || path.starts_with("components/")) {
        return None;
    }
    let stem = Path::new(path).file_stem()?.to_str()?;
    Some(stem.to_string())
}

pub(super) fn extract_exports(path: &Path, rel: &str, hash: &str) -> Result<Vec<ExportMapEntry>> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        for prefix in export_prefixes() {
            if let Some(symbol) = exported_symbol(trimmed, prefix) {
                entries.push(ExportMapEntry {
                    symbol,
                    file: rel.to_string(),
                    hash: hash.to_string(),
                });
            }
        }
    }
    Ok(entries)
}

fn export_prefixes() -> [&'static str; 6] {
    [
        "export function ",
        "export const ",
        "export class ",
        "pub fn ",
        "pub struct ",
        "pub enum ",
    ]
}

fn exported_symbol(line: &str, prefix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?;
    rest.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()
        .filter(|symbol| !symbol.is_empty())
        .map(str::to_string)
}

pub(super) fn file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
