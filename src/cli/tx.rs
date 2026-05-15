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
    Watch {
        tx_id: String,

        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,

        #[arg(long)]
        once: bool,
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
