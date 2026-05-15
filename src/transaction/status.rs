#[derive(Debug, Clone, Copy)]
pub enum TransactionStatus {
    Committed,
    RolledBack,
    BlockedOnHuman,
    Noop,
}

impl TransactionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "COMMITTED",
            Self::RolledBack => "ROLLED_BACK",
            Self::BlockedOnHuman => "BLOCKED_ON_HUMAN",
            Self::Noop => "NOOP",
        }
    }
}
