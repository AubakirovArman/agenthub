mod scan;
mod types;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::agent_dir::AgentPaths;

use scan::*;
pub use types::*;

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
