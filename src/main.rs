mod cli;
mod handlers;
mod project_path;

use std::fs;

use anyhow::Result;
use clap::Parser;

use agenthub::{
    aal, agent_adapter, agent_dir, code_maps, enterprise, shell, skill_registry, team, tui,
    tx_undo, web_dashboard, workspace,
};

use crate::cli::{AalCommands, AgentCommands, Cli, Commands, SkillCommands, WorkspaceCommands};

fn main() {
    if let Err(error) = run() {
        eprintln!("agenthub: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let project_root = project_path::resolve_cli_project(cli.project);

    match cli.command.unwrap_or(Commands::Shell) {
        Commands::Init { force } => {
            agent_dir::init_project(&project_root, force)?;
            println!("initialized AgentHub project at {}", project_root.display());
        }
        Commands::Doctor => handlers::handle_doctor(&project_root)?,
        Commands::Version => handlers::handle_version()?,
        Commands::Shell => shell::run(&project_root)?,
        Commands::Plan {
            request,
            output,
            approval_required,
        } => handlers::handle_plan(
            &project_root,
            &request,
            output.as_deref(),
            approval_required,
        )?,
        Commands::Ask {
            request,
            output,
            approval_required,
        } => handlers::handle_ask(&request, output.as_deref(), approval_required)?,
        Commands::Run { target, no_commit } => {
            handlers::handle_run(&project_root, &target, no_commit)?
        }
        Commands::Undo { target } => {
            enterprise::authorize(&project_root, "transaction.run")?;
            let report = tx_undo::undo(&project_root, &target)?;
            println!(
                "reverted\t{}\t{}\t{}",
                report.tx_id, report.reverted_commit, report.revert_head
            );
        }
        Commands::Tui => {
            enterprise::authorize(&project_root, "transaction.read")?;
            print!("{}", tui::dashboard_text(&project_root)?);
        }
        Commands::Dashboard { output } => {
            enterprise::authorize(&project_root, "transaction.read")?;
            enterprise::authorize(&project_root, "memory.read")?;
            enterprise::authorize(&project_root, "skills.read")?;
            enterprise::authorize(&project_root, "enterprise.policy.read")?;
            let output_dir = resolve_output(&project_root, &output);
            let result = web_dashboard::write_dashboard(&project_root, &output_dir)?;
            team::write_export(
                std::slice::from_ref(&project_root),
                &project_root.join(".agent/reports/team"),
            )?;
            println!("{}", result.index_path.display());
        }
        Commands::Aal { command } => match command {
            AalCommands::Parse { input, output } => {
                let parsed = aal::parse_aal_file(&input)?;
                for diagnostic in &parsed.diagnostics {
                    eprintln!("{}", diagnostic.render());
                }
                if parsed.has_errors() {
                    anyhow::bail!("AAL semantic validation failed");
                }
                let yaml = serde_yaml::to_string(&parsed.spec)?;
                if let Some(output) = output {
                    if let Some(parent) =
                        output.parent().filter(|path| !path.as_os_str().is_empty())
                    {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&output, yaml)?;
                    println!("{}", output.display());
                } else {
                    print!("{yaml}");
                }
            }
        },
        Commands::Tx { command } => handlers::handle_tx(&project_root, command)?,
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Scan { write_maps } => {
                enterprise::authorize(&project_root, "workspace.read")?;
                let scan = workspace::scan(&project_root)?;
                println!("git_repo: {}", scan.git_repo);
                println!(
                    "head: {}",
                    scan.head.unwrap_or_else(|| "<none>".to_string())
                );
                println!("dirty: {}", scan.dirty);
                if write_maps {
                    let maps = code_maps::write(&project_root)?;
                    println!("routes: {}", maps.routes.len());
                    println!("components: {}", maps.components.len());
                    println!("exports: {}", maps.exports.len());
                }
            }
        },
        Commands::Memory { command } => handlers::handle_memory(&project_root, command)?,
        Commands::Skills { command } => match command {
            SkillCommands::List => {
                enterprise::authorize(&project_root, "skills.read")?;
                for manifest in skill_registry::list_available(&project_root)? {
                    println!(
                        "{}\t{}\t{}",
                        manifest.skill.id, manifest.skill.version, manifest.skill.description
                    );
                }
            }
        },
        Commands::Plugins { command } => handlers::handle_plugins(&project_root, command)?,
        Commands::Enterprise { command } => handlers::handle_enterprise(&project_root, command)?,
        Commands::Agents { command } => match command {
            AgentCommands::List => {
                for adapter in agent_adapter::supported_adapters() {
                    println!("{adapter}");
                }
            }
        },
        Commands::Providers { command } => handlers::handle_providers(&project_root, command)?,
        Commands::Config { command } => handlers::handle_config(&project_root, command)?,
    }

    Ok(())
}

fn resolve_output(project_root: &std::path::Path, output: &std::path::Path) -> std::path::PathBuf {
    if output.is_absolute() {
        output.to_path_buf()
    } else {
        project_root.join(output)
    }
}
