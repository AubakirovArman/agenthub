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
    Stats,
    Shell,
    Exec {
        request: String,

        #[arg(long)]
        jsonl: bool,
    },
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

        #[arg(long)]
        no_watch: bool,

        #[arg(long)]
        json: bool,
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
    Ops {
        #[command(subcommand)]
        command: OpsCommands,
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
    Inbox {
        #[command(subcommand)]
        command: Option<MemoryInboxCommands>,
    },
}

#[derive(Debug, Subcommand)]
pub enum MemoryInboxCommands {
    List {
        #[arg(long)]
        all: bool,
    },
    Add {
        note: String,

        #[arg(long, default_value = "core")]
        domain: String,

        #[arg(long, default_value = "architecture_decision")]
        kind: String,
    },
    Approve {
        #[arg(required = true)]
        ids: Vec<String>,
    },
    Reject {
        #[arg(required = true)]
        ids: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum OpsCommands {
    Hosts {
        #[command(subcommand)]
        command: Option<OpsHostCommands>,
    },
    Runbooks {
        #[command(subcommand)]
        command: Option<OpsRunbookCommands>,
    },
    Receipts {
        #[arg(long)]
        host: Option<String>,

        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum OpsHostCommands {
    List,
    Add {
        target: String,

        #[arg(long)]
        alias: Option<String>,

        #[arg(long, default_value = "unknown")]
        trust: String,

        #[arg(long)]
        note: Option<String>,
    },
    Trust {
        target: String,
        trust: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum OpsRunbookCommands {
    List {
        #[arg(long)]
        host: Option<String>,
    },
    Add {
        title: String,

        #[arg(long)]
        host: Option<String>,

        #[arg(long)]
        command: Option<String>,

        #[arg(long)]
        note: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum SkillCommands {
    List,
    Scorecard,
}
