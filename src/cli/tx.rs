use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum TxCommands {
    Status,
    Report {
        tx_id: String,
    },
    Effects {
        tx_id: String,
    },
    Resolve {
        tx_id: String,
        #[arg(long)]
        note: String,
    },
    Resume {
        tx_id: String,
    },
    Retry {
        tx_id: String,
        #[arg(long = "from")]
        from_state: String,
    },
}
