use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "agenthub")]
#[command(about = "Transactional runtime foundation for AI-agent workflows")]
pub struct Cli {
    #[arg(long, global = true, default_value = ".")]
    pub project: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        #[arg(long)]
        force: bool,
    },
    Ask {
        request: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Run {
        spec: PathBuf,

        #[arg(long)]
        no_commit: bool,
    },
    Tx {
        #[command(subcommand)]
        command: TxCommands,
    },
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    Skills {
        #[command(subcommand)]
        command: SkillCommands,
    },
    Agents {
        #[command(subcommand)]
        command: AgentCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum TxCommands {
    Status,
    Report { tx_id: String },
}

#[derive(Debug, Subcommand)]
pub enum WorkspaceCommands {
    Scan {
        #[arg(long)]
        write_maps: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommands {
    Inspect,
}

#[derive(Debug, Subcommand)]
pub enum SkillCommands {
    List,
}

#[derive(Debug, Subcommand)]
pub enum AgentCommands {
    List,
}
