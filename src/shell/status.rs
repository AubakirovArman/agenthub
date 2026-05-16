use std::path::Path;

use anyhow::Result;

use crate::{
    agent_dir, git,
    product_cli::{config, providers},
    workspace,
};

pub(super) fn print(root: &Path) -> Result<()> {
    let mode = workspace::detect_mode(root).mode;
    println!("mode {}", mode.as_str());
    println!("project {}", root.display());
    println!("git {}", if git::is_repo(root) { "ok" } else { "missing" });
    println!(
        "agent {}",
        if root.join(".agent/project.yaml").exists() {
            "ok"
        } else {
            "missing"
        }
    );
    println!("provider {}", config::default_provider(root)?);
    print!("{}", providers::render_status(root)?);
    match latest_tx(root) {
        Ok(tx) => println!("latest_tx {tx}"),
        Err(_) => println!("latest_tx <none>"),
    }
    Ok(())
}

fn latest_tx(root: &Path) -> Result<String> {
    let mut rows = agent_dir::list_transactions(root)?;
    rows.pop()
        .map(|row| row.id)
        .ok_or_else(|| anyhow::anyhow!("no transactions yet"))
}
