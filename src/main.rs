mod cli;

use anyhow::Result;
use clap::Parser;

use agenthub::{
    agent_adapter, agent_dir, code_maps, intent, memory, skill_registry, transaction, workspace,
};

use crate::cli::{
    AgentCommands, Cli, Commands, MemoryCommands, SkillCommands, TxCommands, WorkspaceCommands,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("agenthub: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let project_root = cli.project.canonicalize().unwrap_or(cli.project);

    match cli.command {
        Commands::Init { force } => {
            agent_dir::init_project(&project_root, force)?;
            println!("initialized AgentHub project at {}", project_root.display());
        }
        Commands::Ask { request, output } => {
            let preview = intent::normalize_to_spec(&request);
            if let Some(output) = output {
                let path = intent::write_preview(&preview, &output)?;
                println!("{}", path.display());
            } else {
                print!("{}", preview.agent_spec_yaml);
            }
            if !preview.unknowns.is_empty() {
                eprintln!("unknowns: {}", preview.unknowns.join(", "));
            }
        }
        Commands::Run { spec, no_commit } => {
            let outcome = transaction::run(&project_root, &spec, no_commit)?;
            println!(
                "{} {} ({})",
                outcome.tx_id,
                outcome.status.as_str(),
                outcome.report_path.display()
            );
        }
        Commands::Tx { command } => match command {
            TxCommands::Status => {
                for row in agent_dir::list_transactions(&project_root)? {
                    println!("{}\t{}\t{}", row.id, row.status, row.report_path.display());
                }
            }
            TxCommands::Report { tx_id } => {
                let report = agent_dir::read_report(&project_root, &tx_id)?;
                print!("{report}");
            }
        },
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Scan { write_maps } => {
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
        Commands::Memory { command } => match command {
            MemoryCommands::Inspect => {
                let stats = memory::inspect(&project_root)?;
                println!("committed: {}", stats.committed);
                println!("failed_attempts: {}", stats.failed_attempts);
            }
        },
        Commands::Skills { command } => match command {
            SkillCommands::List => {
                for manifest in skill_registry::list_available(&project_root)? {
                    println!(
                        "{}\t{}\t{}",
                        manifest.skill.id, manifest.skill.version, manifest.skill.description
                    );
                }
            }
        },
        Commands::Agents { command } => match command {
            AgentCommands::List => {
                for adapter in agent_adapter::supported_adapters() {
                    println!("{adapter}");
                }
            }
        },
    }

    Ok(())
}
