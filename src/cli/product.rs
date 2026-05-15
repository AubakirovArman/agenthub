use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ProviderCommands {
    List,
    Status,
    Setup { provider: String },
    Test { provider: String },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    Show,
    Set { key: String, value: String },
}
