use std::path::Path;

use anyhow::Result;
use serde_json::json;

use agenthub::{agent_dir, enterprise, tx_control, tx_explain, tx_watch};

use crate::cli::{EnterpriseCommands, TxCommands};

mod plugin_commands;
mod product_commands;
mod run_commands;
mod run_summary;

pub use plugin_commands::handle_plugins;
pub use product_commands::{handle_config, handle_doctor, handle_providers, handle_version};
pub use run_commands::{handle_ask, handle_plan, handle_run};

pub fn handle_tx(project_root: &Path, command: TxCommands) -> Result<()> {
    match command {
        TxCommands::Status => {
            enterprise::authorize(project_root, "transaction.read")?;
            for row in agent_dir::list_transactions(project_root)? {
                println!("{}\t{}\t{}", row.id, row.status, row.report_path.display());
            }
        }
        TxCommands::Report { tx_id } => {
            enterprise::authorize(project_root, "transaction.read")?;
            print!("{}", agent_dir::read_report(project_root, &tx_id)?);
        }
        TxCommands::Effects { tx_id } => {
            enterprise::authorize(project_root, "transaction.read")?;
            print!("{}", agent_dir::read_effects(project_root, &tx_id)?);
        }
        TxCommands::Explain { tx_id } => {
            enterprise::authorize(project_root, "transaction.read")?;
            print!(
                "{}",
                tx_explain::explain(project_root, &tx_id)?.render_text()
            );
        }
        TxCommands::Watch {
            tx_id,
            interval_ms,
            once,
        } => {
            enterprise::authorize(project_root, "transaction.read")?;
            tx_watch::watch(
                project_root,
                &tx_id,
                tx_watch::WatchOptions { interval_ms, once },
            )?;
        }
        TxCommands::Cancel { tx_id, reason } => {
            enterprise::authorize(project_root, "transaction.run")?;
            let actor = std::env::var("AGENTHUB_ACTOR").unwrap_or_else(|_| "local".to_string());
            let report = tx_control::cancel(project_root, &tx_id, &actor, &reason)?;
            println!(
                "cancel_requested\t{}\t{}\t{}",
                report.tx_id, report.requested_by, report.reason
            );
        }
        TxCommands::Resolve { tx_id, note } => {
            enterprise::authorize(project_root, "transaction.run")?;
            let record = tx_control::resolve(project_root, &tx_id, &note)?;
            println!("resolved\t{}\t{}", record.tx_id, record.ts);
        }
        TxCommands::Resume { tx_id } => {
            enterprise::authorize(project_root, "transaction.run")?;
            let report = tx_control::resume(project_root, &tx_id)?;
            println!(
                "resumed\t{}\t{}\t{}",
                report.tx_id, report.resumed_tx_id, report.status
            );
        }
        TxCommands::Retry { tx_id, from_state } => {
            enterprise::authorize(project_root, "transaction.run")?;
            let plan = tx_control::retry(project_root, &tx_id, &from_state)?;
            println!("{}", plan.retry_plan.display());
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
