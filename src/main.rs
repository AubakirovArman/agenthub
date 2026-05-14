mod cli;
mod handlers;

use anyhow::Result;
use clap::Parser;
use serde_json::json;

use agenthub::{
    agent_adapter, agent_dir, code_maps, enterprise, intent, memory, skill_registry, transaction,
    workspace,
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
        Commands::Ask {
            request,
            output,
            approval_required,
        } => {
            let preview = intent::normalize_to_spec_with_options(
                &request,
                intent::IntentOptions { approval_required },
            );
            if let Some(output) = output {
                let path = intent::write_preview(&preview, &output)?;
                println!("{}", path.display());
            } else {
                print!("{}", preview.agent_spec_yaml);
            }
            if !preview.unknowns.is_empty() {
                eprintln!("unknowns: {}", preview.unknowns.join(", "));
            }
            if !preview.questions.is_empty() {
                eprintln!("questions:");
                for question in &preview.questions {
                    eprintln!("- [{}] {}", question.id, question.question);
                }
            }
        }
        Commands::Run { spec, no_commit } => {
            let actor = enterprise::authorize(&project_root, "transaction.run")?;
            let outcome = match transaction::run(&project_root, &spec, no_commit) {
                Ok(outcome) => outcome,
                Err(error) => {
                    enterprise::record_event(
                        &project_root,
                        &actor,
                        "agenthub.run",
                        "transaction.run",
                        "error",
                        Some(spec.display().to_string()),
                        json!({ "error": error.to_string() }),
                    )?;
                    return Err(error);
                }
            };
            enterprise::record_event(
                &project_root,
                &actor,
                "agenthub.run",
                "transaction.run",
                outcome.status.as_str(),
                Some(spec.display().to_string()),
                json!({ "tx_id": outcome.tx_id }),
            )?;
            println!(
                "{} {} ({})",
                outcome.tx_id,
                outcome.status.as_str(),
                outcome.report_path.display()
            );
        }
        Commands::Tx { command } => match command {
            TxCommands::Status => {
                enterprise::authorize(&project_root, "transaction.read")?;
                for row in agent_dir::list_transactions(&project_root)? {
                    println!("{}\t{}\t{}", row.id, row.status, row.report_path.display());
                }
            }
            TxCommands::Report { tx_id } => {
                enterprise::authorize(&project_root, "transaction.read")?;
                let report = agent_dir::read_report(&project_root, &tx_id)?;
                print!("{report}");
            }
        },
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
        Commands::Memory { command } => match command {
            MemoryCommands::Inspect => {
                enterprise::authorize(&project_root, "memory.read")?;
                let stats = memory::inspect(&project_root)?;
                println!("committed: {}", stats.committed);
                println!("failed_attempts: {}", stats.failed_attempts);
            }
        },
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
    }

    Ok(())
}
