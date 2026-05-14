use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::agent_dir::AgentPaths;

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

pub fn build(root: &Path) -> Result<WorkspaceMaps> {
    let files = source_files(root)?;
    let mut routes = Vec::new();
    let mut components = Vec::new();
    let mut exports = Vec::new();

    for file in files {
        let rel = relative(root, &file);
        let hash = file_hash(&file)?;
        if let Some(route) = route_from_path(&rel) {
            routes.push(RouteMapEntry {
                route,
                file: rel.clone(),
                hash: hash.clone(),
            });
        }
        if let Some(name) = component_from_path(&rel) {
            components.push(ComponentMapEntry {
                name,
                file: rel.clone(),
                hash: hash.clone(),
            });
        }
        exports.extend(extract_exports(&file, &rel, &hash)?);
    }

    routes.sort_by(|a, b| a.route.cmp(&b.route));
    components.sort_by(|a, b| a.name.cmp(&b.name));
    exports.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    Ok(WorkspaceMaps {
        generated_at: Utc::now(),
        routes,
        components,
        exports,
    })
}

pub fn write(root: &Path) -> Result<WorkspaceMaps> {
    let maps = build(root)?;
    let paths = AgentPaths::new(root);
    fs::create_dir_all(&paths.maps).with_context(|| format!("create {}", paths.maps.display()))?;
    fs::write(
        paths.maps.join("routes.map.json"),
        serde_json::to_string_pretty(&maps.routes)?,
    )?;
    fs::write(
        paths.maps.join("components.map.json"),
        serde_json::to_string_pretty(&maps.components)?,
    )?;
    fs::write(
        paths.maps.join("exports.map.json"),
        serde_json::to_string_pretty(&maps.exports)?,
    )?;
    Ok(maps)
}

pub fn read_existing(root: &Path) -> Result<serde_json::Value> {
    let paths = AgentPaths::new(root);
    let routes = read_json(paths.maps.join("routes.map.json").as_path())?;
    let components = read_json(paths.maps.join("components.map.json").as_path())?;
    let exports = read_json(paths.maps.join("exports.map.json").as_path())?;
    Ok(serde_json::json!({
        "routes": routes,
        "components": components,
        "exports": exports,
    }))
}

fn source_files(root: &Path) -> Result<Vec<PathBuf>> {
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

fn route_from_path(path: &str) -> Option<String> {
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

fn component_from_path(path: &str) -> Option<String> {
    if !(path.starts_with("src/components/") || path.starts_with("components/")) {
        return None;
    }
    let stem = Path::new(path).file_stem()?.to_str()?;
    Some(stem.to_string())
}

fn extract_exports(path: &Path, rel: &str, hash: &str) -> Result<Vec<ExportMapEntry>> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        for prefix in [
            "export function ",
            "export const ",
            "export class ",
            "pub fn ",
            "pub struct ",
            "pub enum ",
        ] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(symbol) = rest
                    .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    .next()
                    .filter(|symbol| !symbol.is_empty())
                {
                    entries.push(ExportMapEntry {
                        symbol: symbol.to_string(),
                        file: rel.to_string(),
                        hash: hash.to_string(),
                    });
                }
            }
        }
    }
    Ok(entries)
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::json!([]));
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&content)?)
}

fn file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nextjs_routes_components_and_exports() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let page = dir.path().join("src/app/courses/page.tsx");
        let component = dir.path().join("src/components/CourseCard.tsx");
        fs::create_dir_all(page.parent().unwrap())?;
        fs::create_dir_all(component.parent().unwrap())?;
        fs::write(&page, "export function CoursesPage() { return null }\n")?;
        fs::write(&component, "export const CourseCard = () => null\n")?;

        let maps = build(dir.path())?;

        assert!(maps.routes.iter().any(|route| route.route == "/courses"));
        assert!(maps
            .components
            .iter()
            .any(|component| component.name == "CourseCard"));
        assert!(maps
            .exports
            .iter()
            .any(|export| export.symbol == "CourseCard"));
        Ok(())
    }
}
