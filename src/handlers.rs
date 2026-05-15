use std::path::Path;

use anyhow::Result;
use serde_json::json;

use agenthub::{enterprise, plugin_registry};

use crate::cli::{EnterpriseCommands, PluginCommands};

pub fn handle_plugins(project_root: &Path, command: PluginCommands) -> Result<()> {
    match command {
        PluginCommands::List => {
            enterprise::authorize(project_root, "plugins.read")?;
            for plugin in plugin_registry::list_installed(project_root)? {
                println!(
                    "{}\t{}\t{}\t{}",
                    plugin.id, plugin.version, plugin.trust, plugin.source
                );
            }
        }
        PluginCommands::Inspect { package } => {
            enterprise::authorize(project_root, "plugins.read")?;
            let manifest = plugin_registry::inspect_package(&package)?;
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
        }
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
        } => {
            let actor = enterprise::authorize(project_root, "plugins.install")?;
            let trust = trust.parse()?;
            let result = plugin_registry::install_package(
                project_root,
                &package,
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
        }
    }
    Ok(())
}

pub fn handle_enterprise(project_root: &Path, command: EnterpriseCommands) -> Result<()> {
    match command {
        EnterpriseCommands::Policy => {
            enterprise::authorize(project_root, "enterprise.policy.read")?;
            let source = enterprise::policy_source(project_root)?;
            let policy = enterprise::load_policy(project_root)?;
            println!("source\t{}\t{}", source.mode, source.path);
            println!("enabled\t{}", policy.enterprise.enabled);
            println!("default_role\t{}", policy.enterprise.default_role);
            println!("roles\t{}", policy.enterprise.roles.len());
        }
        EnterpriseCommands::Secrets { name } => {
            enterprise::authorize(project_root, "enterprise.secrets.check")?;
            let checks = if let Some(name) = name {
                vec![enterprise::check_secret(project_root, &name)?]
            } else {
                enterprise::check_required_secrets(project_root)?
            };
            for check in checks {
                println!(
                    "{}\t{}\tallowed:{}\tpresent:{}",
                    check.name, check.provider, check.allowed, check.present
                );
            }
        }
        EnterpriseCommands::Runners => {
            enterprise::authorize(project_root, "enterprise.runners.read")?;
            let inventory = enterprise::runner_inventory(project_root)?;
            println!("default\t{}", inventory.default);
            for runner in inventory.remote {
                println!(
                    "remote\t{}\t{}\t{}",
                    runner.id,
                    runner.endpoint,
                    runner.labels.join(",")
                );
            }
        }
        EnterpriseCommands::ModelRoute { model } => {
            enterprise::authorize(project_root, "enterprise.policy.read")?;
            let route = enterprise::route_model(project_root, &model)?;
            println!(
                "{}\tprivate:{}\trunner:{}",
                route.model, route.private, route.runner
            );
        }
        EnterpriseCommands::Audit { limit } => {
            enterprise::authorize(project_root, "enterprise.audit.read")?;
            for event in enterprise::list_audit(project_root, limit)? {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    event.created_at, event.actor, event.action, event.outcome, event.permission
                );
            }
        }
        EnterpriseCommands::Compliance { output } => {
            let actor = enterprise::authorize(project_root, "enterprise.compliance.generate")?;
            let report = enterprise::generate_compliance_report(project_root, output.as_deref())?;
            enterprise::record_event(
                project_root,
                &actor,
                "agenthub.enterprise.compliance",
                "enterprise.compliance.generate",
                "ok",
                Some(report.path.display().to_string()),
                json!({ "report": report.path.display().to_string() }),
            )?;
            println!("{}", report.path.display());
        }
    }
    Ok(())
}
