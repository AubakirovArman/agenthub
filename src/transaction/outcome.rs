use std::path::PathBuf;

use super::TransactionStatus;

#[derive(Debug, Clone)]
pub struct TransactionOutcome {
    pub tx_id: String,
    pub status: TransactionStatus,
    pub report_path: PathBuf,
}
