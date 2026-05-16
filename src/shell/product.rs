use std::path::Path;

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
        "test" => print!(
            "{}",
            providers::test_provider(root, required(&args, 1, "provider")?)?
        ),
        "diagnose" => print!(
            "{}",
            providers::diagnose_provider(root, required(&args, 1, "provider")?)?
        ),
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
}
