use std::path::Path;

use anyhow::Result;
use serde_json::json;

use agenthub::{enterprise, plugin_registry};

use crate::cli::PluginCommands;

pub fn handle_plugins(project_root: &Path, command: PluginCommands) -> Result<()> {
    match command {
        PluginCommands::List => list(project_root)?,
        PluginCommands::Inspect { package } => inspect(project_root, &package)?,
        PluginCommands::Digest { package } => digest(project_root, &package)?,
        PluginCommands::Scaffold {
            output,
            package_id,
            skill_id,
            description,
            author,
            force,
        } => {
            enterprise::authorize(project_root, "plugins.install")?;
            let manifest = plugin_registry::scaffold_package(
                &output,
                plugin_registry::ScaffoldOptions {
                    package_id,
                    skill_id,
                    description,
                    author,
                    force,
                },
            )?;
            println!("{}", manifest.display());
        }
        PluginCommands::Install {
            package,
            trust,
            allow_untrusted,
            force,
        } => install(project_root, &package, &trust, allow_untrusted, force)?,
    }
    Ok(())
}

fn list(project_root: &Path) -> Result<()> {
    enterprise::authorize(project_root, "plugins.read")?;
    for plugin in plugin_registry::list_installed(project_root)? {
        println!(
            "{}\t{}\t{}\t{}",
            plugin.id, plugin.version, plugin.trust, plugin.source
        );
    }
    Ok(())
}

fn inspect(project_root: &Path, package: &Path) -> Result<()> {
    enterprise::authorize(project_root, "plugins.read")?;
    let manifest = plugin_registry::inspect_package(package)?;
    println!(
        "{}\t{}\t{}",
        manifest.package.id, manifest.package.version, manifest.package.description
    );
    println!("skills: {}", manifest.skills.len());
    println!("workspace_plugins: {}", manifest.workspace_plugins.len());
    println!("verifier_plugins: {}", manifest.verifier_plugins.len());
    println!(
        "signature: {}",
        if manifest.signature.is_some() {
            "present"
        } else {
            "none"
        }
    );
    Ok(())
}

fn digest(project_root: &Path, package: &Path) -> Result<()> {
    enterprise::authorize(project_root, "plugins.read")?;
    println!("sha256\t{}", plugin_registry::package_digest(package)?);
    Ok(())
}

fn install(
    project_root: &Path,
    package: &Path,
    trust: &str,
    allow_untrusted: bool,
    force: bool,
) -> Result<()> {
    let actor = enterprise::authorize(project_root, "plugins.install")?;
    let trust = trust.parse()?;
    let result = plugin_registry::install_package(
        project_root,
        package,
        plugin_registry::InstallOptions {
            trust,
            allow_untrusted,
            force,
        },
    )?;
    enterprise::record_event(
        project_root,
        &actor,
        "agenthub.plugins.install",
        "plugins.install",
        "ok",
        Some(package.display().to_string()),
        json!({ "package": result.package_id, "version": result.package_version }),
    )?;
    println!("installed {} {}", result.package_id, result.package_version);
    for skill in result.skills {
        println!("skill\t{}\t{}\t{}", skill.id, skill.version, skill.target);
    }
    Ok(())
}
