use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::spec::AgentSpec;

use super::{
    ComponentMapEntry, ExportMapEntry, MapContextPolicy, MapContextSelection, MapValidation,
    RouteMapEntry, WorkspaceMapEntries,
};

pub(super) fn for_spec(
    spec: &AgentSpec,
    entries: WorkspaceMapEntries,
    validation: MapValidation,
) -> Result<MapContextSelection> {
    let scope = scope_globs(&spec.scope.allow)?;
    let text = task_text(spec);
    let routes = select_routes(entries.routes, scope.as_ref(), &text);
    let components = select_components(entries.components, scope.as_ref(), &text);
    let exports = select_exports(entries.exports, scope.as_ref(), &text);

    Ok(MapContextSelection {
        routes,
        components,
        exports,
        validation,
        policy: MapContextPolicy {
            map_based: true,
            full_files_included: false,
            selector: "scope_or_task_hint".to_string(),
        },
    })
}

fn select_routes(
    entries: Vec<RouteMapEntry>,
    scope: Option<&GlobSet>,
    text: &str,
) -> Vec<RouteMapEntry> {
    entries
        .into_iter()
        .filter(|entry| in_scope(scope, &entry.file) || text.contains(&entry.route))
        .collect()
}

fn select_components(
    entries: Vec<ComponentMapEntry>,
    scope: Option<&GlobSet>,
    text: &str,
) -> Vec<ComponentMapEntry> {
    entries
        .into_iter()
        .filter(|entry| in_scope(scope, &entry.file) || text_contains(text, &entry.name))
        .collect()
}

fn select_exports(
    entries: Vec<ExportMapEntry>,
    scope: Option<&GlobSet>,
    text: &str,
) -> Vec<ExportMapEntry> {
    entries
        .into_iter()
        .filter(|entry| in_scope(scope, &entry.file) || text_contains(text, &entry.symbol))
        .collect()
}

fn scope_globs(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn in_scope(scope: Option<&GlobSet>, file: &str) -> bool {
    scope.map(|scope| scope.is_match(file)).unwrap_or(true)
}

fn task_text(spec: &AgentSpec) -> String {
    [
        spec.task.id.as_str(),
        spec.task.kind.as_str(),
        spec.task.title.as_deref().unwrap_or_default(),
        spec.task.target.as_deref().unwrap_or_default(),
    ]
    .join(" ")
    .to_ascii_lowercase()
}

fn text_contains(text: &str, value: &str) -> bool {
    text.contains(&value.to_ascii_lowercase())
}
