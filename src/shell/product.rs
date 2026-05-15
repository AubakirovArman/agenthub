use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{
    enterprise,
    product_cli::{
        config, doctor,
        providers::{self},
    },
    team, web_dashboard,
};

pub(super) fn print_doctor(root: &Path) -> Result<()> {
    print!("{}", doctor::inspect(root)?.render());
    Ok(())
}

pub(super) fn handle_providers(root: &Path, args: Option<&str>) -> Result<()> {
    let args = split_args(args.unwrap_or("status"));
    match args.first().copied().unwrap_or("status") {
        "list" => print!("{}", providers::render_list()),
        "status" => print!("{}", providers::render_status(root)?),
        "setup" => print!(
            "{}",
            providers::setup_provider(root, required(&args, 1, "provider")?)?
        ),
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
        other => return Err(anyhow!("unknown providers command `{other}`")),
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
    let output = root.join(".agent/reports/dashboard");
    let result = web_dashboard::write_dashboard(root, &output)?;
    let projects = vec![root.to_path_buf()];
    team::write_export(&projects, &root.join(".agent/reports/team"))?;
    println!("{}", result.index_path.display());
    Ok(())
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
