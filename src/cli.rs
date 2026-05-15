use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod product;
mod tx;

pub use product::{ConfigCommands, ProviderCommands};
pub use tx::TxCommands;

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
    Doctor,
    Version,
    Ask {
        request: String,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        approval_required: bool,
    },
    Run {
        spec: PathBuf,

        #[arg(long)]
        no_commit: bool,
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
}

#[derive(Debug, Subcommand)]
pub enum SkillCommands {
    List,
}

#[derive(Debug, Subcommand)]
pub enum PluginCommands {
    List,
    Inspect {
        package: PathBuf,
    },
    Digest {
        package: PathBuf,
    },
    Scaffold {
        output: PathBuf,

        #[arg(long)]
        package_id: String,

        #[arg(long)]
        skill_id: String,

        #[arg(long)]
        description: String,

        #[arg(long)]
        author: Option<String>,

        #[arg(long)]
        force: bool,
    },
    Install {
        package: PathBuf,

        #[arg(long, default_value = "local")]
        trust: String,

        #[arg(long)]
        allow_untrusted: bool,

        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum EnterpriseCommands {
    Policy,
    PolicyServer {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: String,

        #[arg(long)]
        policy: Option<PathBuf>,

        #[arg(long, default_value = "AGENTHUB_POLICY_TOKEN")]
        token_env: String,

        #[arg(long)]
        once: bool,
    },
    Secrets {
        name: Option<String>,
    },
    Runners,
    ModelRoute {
        model: String,
    },
    Audit {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Compliance {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentCommands {
    List,
}
