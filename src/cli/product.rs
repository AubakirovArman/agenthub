use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ProviderCommands {
    List,
    Status,
    Setup {
        provider: String,
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
