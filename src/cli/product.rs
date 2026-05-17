use std::path::PathBuf;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ProviderCommands {
    List,
    Status {
        #[arg(long)]
        json: bool,
    },
    Recovery {
        #[arg(long)]
        json: bool,
    },
    Setup {
        provider: String,
    },
    Test {
        provider: String,
    },
    Diagnose {
        provider: String,
    },
    Unblock {
        provider: String,
    },
    RcUnblock {
        provider: String,

        #[arg(long)]
        from_file: Option<PathBuf>,

        #[arg(long)]
        from_env: Option<String>,

        #[arg(long)]
        stdin: bool,

        #[arg(long)]
        target: Option<PathBuf>,

        #[arg(long)]
        skip_provider_dogfood: bool,

        #[arg(long)]
        no_check: bool,
    },
    PreflightKey {
        provider: String,

        #[arg(long)]
        from_file: Option<PathBuf>,

        #[arg(long)]
        from_env: Option<String>,

        #[arg(long)]
        stdin: bool,
    },
    RotateKey {
        provider: String,

        #[arg(long)]
        from_file: Option<PathBuf>,

        #[arg(long)]
        from_env: Option<String>,

        #[arg(long)]
        stdin: bool,

        #[arg(long)]
        target: Option<PathBuf>,

        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        no_test: bool,
    },
    Set {
        role: String,
        provider: String,
    },
    Fallback {
        role: String,
        providers: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    Show,
    Set { key: String, value: String },
}

#[derive(Debug, Subcommand)]
pub enum EcosystemCommands {
    Status {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum OpenCommands {
    Dashboard,
    Report { tx_id: String },
}
