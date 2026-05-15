use std::path::Path;

use anyhow::{bail, Result};

use agenthub::product_cli::{config, doctor, providers, version};

use crate::cli::{ConfigCommands, ProviderCommands};

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
        ProviderCommands::Test { provider } => {
            print!("{}", providers::test_provider(project_root, &provider)?);
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
