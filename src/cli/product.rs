use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ProviderCommands {
    List,
    Status,
    Setup {
        provider: String,
    },
    Add {
        provider: String,

        #[arg(long)]
        name: String,

        #[arg(long)]
        url: String,

        #[arg(long)]
        model: Option<String>,

        #[arg(long = "api-key-env")]
        api_key_env: Option<String>,
    },
    Test {
        provider: String,
    },
    Diagnose {
        provider: String,
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
pub enum OpenCommands {
    Dashboard,
    Report { tx_id: String },
}
