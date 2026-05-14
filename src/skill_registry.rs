use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub skill: SkillMeta,
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
    #[serde(default)]
    pub requires: BTreeMap<String, Value>,
    #[serde(default)]
    pub provides: BTreeMap<String, Value>,
    #[serde(default)]
    pub policies: BTreeMap<String, Value>,
    #[serde(default)]
    pub verifiers: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub common_errors: Vec<String>,
    #[serde(default)]
    pub prompt_fragments: Vec<String>,
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub id: String,
    pub version: String,
    pub description: String,
}

pub fn list_available(project_root: &Path) -> Result<Vec<SkillManifest>> {
    let mut manifests = Vec::new();
    let skills_root = project_root.join("skills");
    if !skills_root.exists() {
        return Ok(manifests);
    }

    for path in find_skill_files(&skills_root)? {
        manifests.push(load_manifest(&path)?);
    }
    manifests.sort_by(|a, b| a.skill.id.cmp(&b.skill.id));
    Ok(manifests)
}

pub fn load_requested(project_root: &Path, requested: &[String]) -> Result<Vec<SkillManifest>> {
    if requested.is_empty() {
        return Ok(Vec::new());
    }

    let available = list_available(project_root)?
        .into_iter()
        .map(|manifest| (manifest.skill.id.clone(), manifest))
        .collect::<BTreeMap<_, _>>();

    let mut loaded = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    for skill_id in requested {
        load_with_dependencies(skill_id, &available, &mut loaded, &mut visiting)?;
    }

    Ok(loaded.into_values().collect())
}

fn load_with_dependencies(
    skill_id: &str,
    available: &BTreeMap<String, SkillManifest>,
    loaded: &mut BTreeMap<String, SkillManifest>,
    visiting: &mut BTreeSet<String>,
) -> Result<()> {
    if loaded.contains_key(skill_id) {
        return Ok(());
    }
    if !visiting.insert(skill_id.to_string()) {
        return Err(anyhow!("cyclic skill dependency involving {skill_id}"));
    }

    let manifest = available
        .get(skill_id)
        .ok_or_else(|| anyhow!("requested skill is not installed: {skill_id}"))?;
    for dependency in &manifest.dependencies {
        load_with_dependencies(dependency, available, loaded, visiting)?;
    }

    visiting.remove(skill_id);
    loaded.insert(skill_id.to_string(), manifest.clone());
    Ok(())
}

fn find_skill_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit_dirs(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            visit_dirs(&path, files)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("skill.yaml") {
            files.push(path);
        }
    }
    Ok(())
}

fn load_manifest(path: &Path) -> Result<SkillManifest> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut manifest: SkillManifest =
        serde_yaml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    manifest.source_path = Some(path.to_path_buf());
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_dependencies() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_skill(dir.path(), "base", "base.skill", &[])?;
        write_skill(dir.path(), "feature", "feature.skill", &["base.skill"])?;

        let loaded = load_requested(dir.path(), &["feature.skill".to_string()])?;
        let ids = loaded
            .into_iter()
            .map(|manifest| manifest.skill.id)
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["base.skill", "feature.skill"]);
        Ok(())
    }

    fn write_skill(root: &Path, dir_name: &str, id: &str, dependencies: &[&str]) -> Result<()> {
        let dir = root.join("skills").join(dir_name);
        fs::create_dir_all(&dir)?;
        let dependency_yaml = dependencies
            .iter()
            .map(|dependency| format!("  - {dependency}\n"))
            .collect::<String>();
        let dependencies_block = if dependencies.is_empty() {
            String::new()
        } else {
            format!("dependencies:\n{dependency_yaml}")
        };
        fs::write(
            dir.join("skill.yaml"),
            format!(
                "skill:\n  id: {id}\n  version: 1.0.0\n  description: test skill\n{dependencies_block}"
            ),
        )?;
        Ok(())
    }
}
