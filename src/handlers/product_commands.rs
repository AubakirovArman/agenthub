use std::{io::Read, path::Path};

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
        ProviderCommands::Test { provider } => {
            let report = providers::test_provider(project_root, &provider)?;
            print!("{}", report);
            if providers::test_report_failed(&report) {
                bail!("provider test failed for `{provider}`");
            }
        }
        ProviderCommands::Diagnose { provider } => {
            print!("{}", providers::diagnose_provider(project_root, &provider)?);
        }
        ProviderCommands::Unblock { provider } => {
            print!("{}", providers::unblock_provider(project_root, &provider)?);
        }
        ProviderCommands::RcUnblock {
            provider,
            from_file,
            from_env,
            stdin,
            target,
            skip_provider_dogfood,
            no_check,
        } => {
            let stdin_value = if stdin {
                let mut value = String::new();
                std::io::stdin().read_to_string(&mut value)?;
                Some(value)
            } else {
                None
            };
            let rotate_key =
                if from_file.is_some() || from_env.is_some() || stdin || target.is_some() {
                    Some(providers::KeyRotationOptions {
                        from_file,
                        from_env,
                        stdin_value,
                        target,
                        dry_run: false,
                        test_after_install: false,
                    })
                } else {
                    None
                };
            let result = providers::rc_unblock_provider(
                project_root,
                &provider,
                providers::RcUnblockOptions {
                    skip_provider_dogfood,
                    no_check,
                    rotate_key,
                },
            )?;
            print!("{}", result.output);
            if result.failed {
                bail!("provider RC unblock failed for `{provider}`");
            }
        }
        ProviderCommands::PreflightKey {
            provider,
            from_file,
            from_env,
            stdin,
        } => {
            let stdin_value = if stdin {
                let mut value = String::new();
                std::io::stdin().read_to_string(&mut value)?;
                Some(value)
            } else {
                None
            };
            let result = providers::preflight_provider_key(
                project_root,
                &provider,
                providers::KeyPreflightOptions {
                    from_file,
                    from_env,
                    stdin_value,
                },
            )?;
            print!("{}", result.output);
            if result.provider_test_failed {
                bail!("provider key preflight failed for `{provider}`");
            }
        }
        ProviderCommands::RotateKey {
            provider,
            from_file,
            from_env,
            stdin,
            target,
            dry_run,
            no_test,
        } => {
            let stdin_value = if stdin {
                let mut value = String::new();
                std::io::stdin().read_to_string(&mut value)?;
                Some(value)
            } else {
                None
            };
            let result = providers::rotate_provider_key(
                project_root,
                &provider,
                providers::KeyRotationOptions {
                    from_file,
                    from_env,
                    stdin_value,
                    target,
                    dry_run,
                    test_after_install: !no_test,
                },
            )?;
            print!("{}", result.output);
            if result.provider_test_failed {
                bail!("provider key rotation test failed for `{provider}`");
            }
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
