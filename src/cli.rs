use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod admin;
mod product;
mod tx;

pub use admin::{AgentCommands, EnterpriseCommands, PluginCommands};
pub use product::{ConfigCommands, ProviderCommands};
pub use tx::TxCommands;

#[derive(Debug, Parser)]
#[command(name = "agenthub")]
#[command(about = "Transactional runtime foundation for AI-agent workflows")]
pub struct Cli {
    #[arg(long, global = true, default_value = ".")]
    pub project: PathBuf,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        #[arg(long)]
        force: bool,
    },
    Doctor,
    Version,
    Shell,
    Plan {
        request: String,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        approval_required: bool,
    },
    Ask {
        request: String,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        approval_required: bool,
    },
    Run {
        target: String,

        #[arg(long)]
        no_commit: bool,
    },
    Undo {
        #[arg(default_value = "last")]
        target: String,
    },
    Tui,
    Dashboard {
        #[arg(short, long, default_value = ".agent/reports/dashboard")]
        output: PathBuf,
    },
    Aal {
        #[command(subcommand)]
        command: AalCommands,
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
    Plugins {
        #[command(subcommand)]
        command: PluginCommands,
    },
    Enterprise {
        #[command(subcommand)]
        command: EnterpriseCommands,
    },
    Agents {
        #[command(subcommand)]
        command: AgentCommands,
    },
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum AalCommands {
    Parse {
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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
    Summary,
    Audit,
}

#[derive(Debug, Subcommand)]
pub enum SkillCommands {
    List,
    Scorecard,
}
