use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::{
    enterprise, local_server,
    product_cli::{
        config, doctor, open,
        providers::{self},
    },
};

pub(super) fn print_doctor(root: &Path) -> Result<()> {
    print!("{}", doctor::inspect(root)?.render());
    Ok(())
}

pub(super) fn handle_providers(root: &Path, args: Option<&str>) -> Result<()> {
    let args = split_args(args.unwrap_or("status"));
    if args.first().copied().unwrap_or("status") == "status" && args.len() == 1 {
        print!("{}", providers::render_wizard(root)?);
        return Ok(());
    }
    match args.first().copied().unwrap_or("status") {
        "list" => print!("{}", providers::render_list()),
        "status" => print!("{}", providers::render_status(root)?),
        "setup" => print!(
            "{}",
            providers::setup_provider(root, required(&args, 1, "provider")?)?
        ),
        "add" => {
            return Err(anyhow!(
            "custom provider profiles are disabled in API-native mode; use `deepseek` or `kimi`"
        ))
        }
        "test" => {
            let provider = required(&args, 1, "provider")?;
            let report = providers::test_provider(root, provider)?;
            print!("{report}");
            if providers::test_report_failed(&report) {
                return Err(anyhow!("provider test failed for `{provider}`"));
            }
        }
        "diagnose" => print!(
            "{}",
            providers::diagnose_provider(root, required(&args, 1, "provider")?)?
        ),
        "unblock" => print!(
            "{}",
            providers::unblock_provider(root, required(&args, 1, "provider")?)?
        ),
        "preflight-key" => {
            let (provider, options) = preflight_key_options_from_args(&args)?;
            let result = providers::preflight_provider_key(root, &provider, options)?;
            print!("{}", result.output);
            if result.provider_test_failed {
                return Err(anyhow!("provider key preflight failed"));
            }
        }
        "rc-unblock" => {
            let (provider, options) = rc_unblock_options_from_args(&args)?;
            let result = providers::rc_unblock_provider(root, &provider, options)?;
            print!("{}", result.output);
            if result.failed {
                return Err(anyhow!("provider RC unblock failed"));
            }
        }
        "set" => print!(
            "{}",
            providers::set_role_provider(
                root,
                required(&args, 1, "role")?,
                required(&args, 2, "provider")?
            )?
        ),
        "fallback" => {
            let role = required(&args, 1, "role")?;
            let fallback = args
                .iter()
                .skip(2)
                .map(|value| value.to_string())
                .collect::<Vec<_>>();
            print!("{}", providers::set_role_fallback(root, role, &fallback)?);
        }
        other if provider_exists(root, other)? => {
            print!("{}", providers::setup_provider(root, other)?)
        }
        other => {
            return Err(anyhow!(
            "unknown providers command `{other}`; use `/providers setup {other}` or `/providers`"
        ))
        }
    }
    Ok(())
}

pub(super) fn handle_config(root: &Path, args: Option<&str>) -> Result<()> {
    let args = split_args(args.unwrap_or("show"));
    match args.first().copied().unwrap_or("show") {
        "show" => print!("{}", config::render_show(root)?),
        "set" => {
            let key = required(&args, 1, "key")?;
            let value = required(&args, 2, "value")?;
            let path = config::set_value(root, key, value)?;
            println!("set\t{key}\t{value}\t{}", path.display());
        }
        other => return Err(anyhow!("unknown config command `{other}`")),
    }
    Ok(())
}

pub(super) fn open_dashboard(root: &Path) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    enterprise::authorize(root, "memory.read")?;
    enterprise::authorize(root, "skills.read")?;
    enterprise::authorize(root, "enterprise.policy.read")?;
    let result = open::dashboard(root)?;
    println!("{}", result.path.display());
    Ok(())
}

pub(super) fn serve_dashboard(root: &Path, args: Option<&str>) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    enterprise::authorize(root, "memory.read")?;
    enterprise::authorize(root, "skills.read")?;
    enterprise::authorize(root, "enterprise.policy.read")?;
    let addr = args
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("127.0.0.1:4317")
        .to_string();
    local_server::serve(
        root,
        local_server::ServerOptions {
            addr,
            output_dir: root.join(".agent/reports/dashboard"),
            refresh_ms: 3000,
            once: false,
        },
    )
}

fn split_args(value: &str) -> Vec<&str> {
    value.split_whitespace().collect()
}

fn required<'a>(args: &'a [&str], index: usize, name: &str) -> Result<&'a str> {
    args.get(index)
        .copied()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing {name}"))
}

fn preflight_key_options_from_args(
    args: &[&str],
) -> Result<(String, providers::KeyPreflightOptions)> {
    let provider = required(args, 1, "provider")?.to_string();
    let mut from_file = None;
    let mut from_env = None;
    let mut index = 2;
    while index < args.len() {
        match args[index] {
            "--from-file" => {
                index += 1;
                from_file = Some(PathBuf::from(required(args, index, "from-file")?));
            }
            "--from-env" => {
                index += 1;
                from_env = Some(required(args, index, "from-env")?.to_string());
            }
            "--stdin" => {
                return Err(anyhow!(
                    "`/providers preflight-key` does not support --stdin; use the CLI command"
                ))
            }
            other => return Err(anyhow!("unknown preflight-key option `{other}`")),
        }
        index += 1;
    }
    Ok((
        provider,
        providers::KeyPreflightOptions {
            from_file,
            from_env,
            stdin_value: None,
        },
    ))
}

fn rc_unblock_options_from_args(args: &[&str]) -> Result<(String, providers::RcUnblockOptions)> {
    let provider = required(args, 1, "provider")?.to_string();
    let mut from_file = None;
    let mut from_env = None;
    let mut target = None;
    let mut skip_provider_dogfood = false;
    let mut no_check = false;
    let mut index = 2;
    while index < args.len() {
        match args[index] {
            "--from-file" => {
                index += 1;
                from_file = Some(PathBuf::from(required(args, index, "from-file")?));
            }
            "--from-env" => {
                index += 1;
                from_env = Some(required(args, index, "from-env")?.to_string());
            }
            "--target" => {
                index += 1;
                target = Some(PathBuf::from(required(args, index, "target")?));
            }
            "--skip-provider-dogfood" => skip_provider_dogfood = true,
            "--no-check" => no_check = true,
            "--stdin" => {
                return Err(anyhow!(
                    "`/providers rc-unblock` does not support --stdin; use the CLI command"
                ))
            }
            other => return Err(anyhow!("unknown rc-unblock option `{other}`")),
        }
        index += 1;
    }
    let rotate_key = if from_file.is_some() || from_env.is_some() || target.is_some() {
        Some(providers::KeyRotationOptions {
            from_file,
            from_env,
            target,
            test_after_install: false,
            ..Default::default()
        })
    } else {
        None
    };
    Ok((
        provider,
        providers::RcUnblockOptions {
            skip_provider_dogfood,
            no_check,
            rotate_key,
        },
    ))
}

fn provider_exists(root: &Path, provider: &str) -> Result<bool> {
    Ok(providers::statuses(root)?
        .into_iter()
        .any(|status| status.info.id == provider))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn providers_rejects_internal_command_as_setup_shorthand() -> Result<()> {
        let dir = tempfile::tempdir()?;

        let error = handle_providers(dir.path(), Some("command")).unwrap_err();

        assert!(error.to_string().contains("unknown providers command"));
        Ok(())
    }

    #[test]
    fn rc_unblock_options_parse_rotation_source() -> Result<()> {
        let args = split_args("rc-unblock kimi --from-file ./new.key --no-check");

        let (provider, options) = rc_unblock_options_from_args(&args)?;

        assert_eq!(provider, "kimi");
        assert!(options.no_check);
        assert!(options.rotate_key.is_some());
        Ok(())
    }

    #[test]
    fn preflight_key_options_parse_rotation_source() -> Result<()> {
        let args = split_args("preflight-key kimi --from-file ./new.key");

        let (provider, options) = preflight_key_options_from_args(&args)?;

        assert_eq!(provider, "kimi");
        assert!(options.from_file.is_some());
        Ok(())
    }
}
