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
        return provider_wizard(root);
    }
    match args.first().copied().unwrap_or("status") {
        "list" => print!("{}", providers::render_list()),
        "status" => print!("{}", providers::render_status(root)?),
        "setup" => print!(
            "{}",
            providers::setup_provider(root, required(&args, 1, "provider")?)?
        ),
        "add" => {
            let provider = required(&args, 1, "provider")?;
            if provider != "openai-http" {
                return Err(anyhow!(
                    "only `openai-http` provider profiles are supported"
                ));
            }
            print!(
                "{}",
                providers::add_openai_http(
                    root,
                    required(&args, 2, "name")?,
                    required(&args, 3, "url")?,
                    args.get(4).copied(),
                    args.get(5).copied()
                )?
            );
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
        other => return Err(anyhow!("unknown providers command `{other}`")),
    }
    Ok(())
}

fn provider_wizard(root: &Path) -> Result<()> {
    println!("Providers");
    print!("{}", providers::render_status(root)?);
    println!("Actions:");
    println!("  /providers setup codex|kimi|gemini|command|openai-http");
    println!("  /providers add openai-http <name> <url> [model] [api_key_env]");
    println!("  /providers diagnose <provider>");
    println!("  /providers test <provider>");
    println!("  /providers set executor <provider>");
    println!("  /providers fallback reviewer gemini kimi command");
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
