use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod admin;
mod product;
mod tx;

pub use admin::{AgentCommands, EnterpriseCommands, PluginCommands};
pub use product::{ConfigCommands, OpenCommands, ProviderCommands};
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
    Tui {
        #[arg(long)]
        live: bool,

        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,

        #[arg(long)]
        once: bool,
    },
    Open {
        #[command(subcommand)]
        command: OpenCommands,
    },
    Dashboard {
        #[arg(short, long, default_value = ".agent/reports/dashboard")]
        output: PathBuf,
    },
    Serve {
        #[arg(long, default_value = "127.0.0.1:4317")]
        addr: String,

        #[arg(short, long, default_value = ".agent/reports/dashboard")]
        output: PathBuf,

        #[arg(long, default_value_t = 3000)]
        refresh_ms: u64,

        #[arg(long)]
        once: bool,
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
    Format {
        input: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        check: bool,
    },
    Check {
        input: PathBuf,

        #[arg(long)]
        expected_dir: Option<PathBuf>,

        #[arg(long)]
        write_expected: bool,
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
