use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;

use crate::agent_dir::ensure_runtime_dirs;
use crate::plugin_registry::governance;
use crate::plugin_registry::lock::{
    upsert_skill_locks, write_plugin_lock, LockedPlugin, LockedSkill, LockedVerifierPlugin,
    LockedWorkspacePlugin,
};
use crate::plugin_registry::package::{manifest_path, read_skill_manifest, validate_package_files};
use crate::plugin_registry::signature;
use crate::plugin_registry::types::{PluginManifest, PluginTrust};

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub trust: PluginTrust,
    pub allow_untrusted: bool,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub package_id: String,
    pub package_version: String,
    pub skills: Vec<LockedSkill>,
}

pub fn inspect_package(path: &Path) -> Result<PluginManifest> {
    let manifest_path = manifest_path(path)?;
    let package_root = manifest_path.parent().expect("manifest has parent");
    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: PluginManifest = serde_yaml::from_str(&content)
        .with_context(|| format!("parse {}", manifest_path.display()))?;
    manifest.validate()?;
    validate_package_files(package_root, &manifest)?;
    signature::verify_package(package_root, &manifest_path, &manifest)?;
    Ok(manifest)
}

pub fn install_package(
    project_root: &Path,
    package_path: &Path,
    options: InstallOptions,
) -> Result<InstallResult> {
    if options.trust == PluginTrust::Untrusted && !options.allow_untrusted {
        return Err(anyhow!(
            "untrusted plugin install requires --allow-untrusted"
        ));
    }

    let paths = ensure_runtime_dirs(project_root)?;
    let manifest_path = manifest_path(package_path)?;
    let package_root = manifest_path.parent().expect("manifest has parent");
    let manifest = inspect_package(package_path)?;
    let signature = signature::verify_package(package_root, &manifest_path, &manifest)?;
    if options.trust == PluginTrust::Trusted && !signature.verified {
        return Err(anyhow!(
            "trusted plugin install requires a verified cryptographic signature"
        ));
    }
    governance::enforce_install(&manifest, options.trust, options.allow_untrusted)?;
    let (_, scorecard_path) = governance::write_scorecard(
        project_root,
        package_root,
        &manifest,
        options.trust,
        signature.verified,
    )?;
    let mut locked_skills = Vec::new();

    for skill in &manifest.skills {
        let source = package_root.join(&skill.path);
        let skill_manifest = read_skill_manifest(&source)?;
        let target_dir = project_root.join("skills").join(&skill_manifest.skill.id);
        let target = target_dir.join("skill.yaml");
        if target.exists() && !options.force {
            let existing = read_skill_manifest(&target)?;
            if existing.skill.version != skill_manifest.skill.version {
                return Err(anyhow!(
                    "skill {} already installed at version {}; use --force to replace with {}",
                    existing.skill.id,
                    existing.skill.version,
                    skill_manifest.skill.version
                ));
            }
        }
        fs::create_dir_all(&target_dir)
            .with_context(|| format!("create {}", target_dir.display()))?;
        fs::copy(&source, &target)
            .with_context(|| format!("copy {} to {}", source.display(), target.display()))?;
        locked_skills.push(LockedSkill {
            id: skill_manifest.skill.id,
            version: skill_manifest.skill.version,
            target: target
                .strip_prefix(project_root)
                .unwrap_or(&target)
                .display()
                .to_string(),
        });
    }

    let lock = LockedPlugin {
        id: manifest.package.id.clone(),
        version: manifest.package.version.clone(),
        source: manifest_path.display().to_string(),
        trust: options.trust.to_string(),
        installed_at: Utc::now(),
        skills: locked_skills.clone(),
        workspace_plugins: manifest
            .workspace_plugins
            .iter()
            .map(|plugin| plugin.id.clone())
            .collect(),
        verifier_plugins: manifest
            .verifier_plugins
            .iter()
            .map(|plugin| plugin.id.clone())
            .collect(),
        workspace_plugin_metadata: manifest
            .workspace_plugins
            .iter()
            .map(|plugin| LockedWorkspacePlugin {
                id: plugin.id.clone(),
                kind: plugin.kind.clone(),
                profile: plugin.profile.clone(),
                schema_path: plugin
                    .schema_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
                capabilities: plugin.capabilities.clone(),
            })
            .collect(),
        verifier_plugin_metadata: manifest
            .verifier_plugins
            .iter()
            .map(|plugin| LockedVerifierPlugin {
                id: plugin.id.clone(),
                command: plugin.command.clone(),
                profiles: plugin.profiles.clone(),
                artifact_globs: plugin.artifact_globs.clone(),
                timeout_secs: plugin.timeout_secs,
            })
            .collect(),
        signature: manifest.signature.clone(),
        signature_verified: signature.verified,
        permissions: manifest.governance.permissions.clone(),
        publisher: manifest.governance.publisher.clone(),
        review: manifest.governance.review.clone(),
        compatibility: manifest.governance.compatibility.clone(),
        advisories: manifest.governance.advisories.clone(),
        scorecard_path: Some(scorecard_path),
    };

    write_plugin_lock(project_root, lock)?;
    upsert_skill_locks(project_root, &locked_skills, &manifest.package.id)?;
    fs::create_dir_all(&paths.plugins)
        .with_context(|| format!("create {}", paths.plugins.display()))?;

    Ok(InstallResult {
        package_id: manifest.package.id,
        package_version: manifest.package.version,
        skills: locked_skills,
    })
}
