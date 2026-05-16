use std::path::Path;

use anyhow::{bail, Result};

use agenthub::product_cli::{config, doctor, open, providers, version};

use crate::cli::{ConfigCommands, OpenCommands, ProviderCommands};

pub fn handle_doctor(project_root: &Path) -> Result<()> {
    let report = doctor::inspect(project_root)?;
    print!("{}", report.render());
    if report.has_errors() {
        bail!("doctor found blocking errors");
    }
    Ok(())
}

pub fn handle_version() -> Result<()> {
    println!("agenthub {}", version());
    Ok(())
}

pub fn handle_providers(project_root: &Path, command: ProviderCommands) -> Result<()> {
    match command {
        ProviderCommands::List => print!("{}", providers::render_list()),
        ProviderCommands::Status => print!("{}", providers::render_status(project_root)?),
        ProviderCommands::Setup { provider } => {
            print!("{}", providers::setup_provider(project_root, &provider)?);
        }
        ProviderCommands::Add {
            provider,
            name,
            url,
            model,
            api_key_env,
        } => {
            if provider != "openai-http" {
                anyhow::bail!("only `openai-http` provider profiles are supported");
            }
            print!(
                "{}",
                providers::add_openai_http(
                    project_root,
                    &name,
                    &url,
                    model.as_deref(),
                    api_key_env.as_deref()
                )?
            );
        }
        ProviderCommands::Test { provider } => {
            print!("{}", providers::test_provider(project_root, &provider)?);
        }
        ProviderCommands::Diagnose { provider } => {
            print!("{}", providers::diagnose_provider(project_root, &provider)?);
        }
        ProviderCommands::Set { role, provider } => {
            print!(
                "{}",
                providers::set_role_provider(project_root, &role, &provider)?
            );
        }
        ProviderCommands::Fallback {
            role,
            providers: items,
        } => {
            print!(
                "{}",
                providers::set_role_fallback(project_root, &role, &items)?
            );
        }
    }
    Ok(())
}

pub fn handle_config(project_root: &Path, command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => print!("{}", config::render_show(project_root)?),
        ConfigCommands::Set { key, value } => {
            let path = config::set_value(project_root, &key, &value)?;
            println!("set\t{}\t{}\t{}", key, value, path.display());
        }
    }
    Ok(())
}

pub fn handle_open(project_root: &Path, command: OpenCommands) -> Result<()> {
    match command {
        OpenCommands::Dashboard => {
            authorize_dashboard(project_root)?;
            print_open(open::dashboard(project_root)?);
        }
        OpenCommands::Report { tx_id } => {
            agenthub::enterprise::authorize(project_root, "transaction.read")?;
            print_open(open::report(project_root, &tx_id)?);
        }
    }
    Ok(())
}

fn authorize_dashboard(project_root: &Path) -> Result<()> {
    agenthub::enterprise::authorize(project_root, "transaction.read")?;
    agenthub::enterprise::authorize(project_root, "memory.read")?;
    agenthub::enterprise::authorize(project_root, "skills.read")?;
    agenthub::enterprise::authorize(project_root, "enterprise.policy.read")?;
    Ok(())
}

fn print_open(result: open::OpenResult) {
    println!(
        "open\t{}\t{}\tlaunched:{}",
        result.kind,
        result.path.display(),
        result.launched
    );
}
