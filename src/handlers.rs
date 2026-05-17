use std::path::Path;

use anyhow::Result;
use serde_json::json;

use agenthub::enterprise;

use crate::cli::EnterpriseCommands;

mod aal_commands;
mod memory_commands;
mod ops_commands;
mod plugin_commands;
mod product_commands;
mod run_commands;
mod run_summary;
mod tx_commands;

pub use aal_commands::handle_aal;
pub use plugin_commands::handle_plugins;
pub use product_commands::{
    handle_config, handle_doctor, handle_open, handle_providers, handle_version,
};
pub use run_commands::{handle_ask, handle_plan, handle_run};
pub use tx_commands::handle_tx;

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
        EnterpriseCommands::PolicyServer {
            bind,
            policy,
            token_env,
            once,
        } => {
            let actor = enterprise::authorize(project_root, "enterprise.policy.serve")?;
            let policy_path =
                policy.unwrap_or_else(|| project_root.join(".agent/enterprise/policy.yaml"));
            let token = std::env::var(&token_env)
                .ok()
                .filter(|value| !value.is_empty());
            enterprise::record_event(
                project_root,
                &actor,
                "agenthub.enterprise.policy_server",
                "enterprise.policy.serve",
                "started",
                Some(policy_path.display().to_string()),
                json!({ "bind": bind.clone(), "token_env": token_env.clone() }),
            )?;
            let server = enterprise::PolicyServer::bind(enterprise::PolicyServerConfig {
                bind,
                policy_path,
                token,
                once,
            })?;
            println!("policy_server\t{}", server.local_addr()?);
            let result = server.serve()?;
            println!("requests\t{}", result.requests);
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
pub use memory_commands::handle_memory;
pub use ops_commands::handle_ops;
