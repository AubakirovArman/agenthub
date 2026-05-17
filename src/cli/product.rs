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
    Select {
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
    InspectKey {
        provider: String,

        #[arg(long)]
        json: bool,

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
pub enum ReadinessCommands {
    Completion {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        no_refresh: bool,
    },
    Next {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        no_refresh: bool,
    },
    Audit {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        no_refresh: bool,
    },
    Blockers {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        no_refresh: bool,
    },
    Checklist {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        no_refresh: bool,
    },
    Evidence {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        check: bool,

        #[arg(long)]
        no_refresh: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum OpenCommands {
    Dashboard,
    Report { tx_id: String },
}
