mod agent_dir;
mod cli;
mod command_runner;
mod diff_guard;
mod git;
mod journal;
mod memory;
mod report;
mod spec;
mod transaction;
mod verifier;
mod workspace;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Commands, MemoryCommands, TxCommands, WorkspaceCommands};

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
            WorkspaceCommands::Scan => {
                let scan = workspace::scan(&project_root)?;
                println!("git_repo: {}", scan.git_repo);
                println!("head: {}", scan.head.unwrap_or_else(|| "<none>".to_string()));
                println!("dirty: {}", scan.dirty);
            }
        },
        Commands::Memory { command } => match command {
            MemoryCommands::Inspect => {
                let stats = memory::inspect(&project_root)?;
                println!("committed: {}", stats.committed);
                println!("failed_attempts: {}", stats.failed_attempts);
            }
        },
    }

    Ok(())
}

