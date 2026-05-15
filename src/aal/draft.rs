use serde::{Deserialize, Serialize};

use crate::spec::{RouteCheckSpec, TransactionSpec};

#[derive(Default)]
pub(crate) struct Draft {
    pub version: Option<String>,
    pub name: Option<String>,
    pub goal: Option<String>,
    pub workspace: Option<String>,
    pub topology: Option<String>,
    pub imports: Vec<AalImport>,
    pub skills: Vec<String>,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub rules: Vec<String>,
    pub execution_commands: Vec<String>,
    pub verify_commands: Vec<String>,
    pub verify_profile: Option<String>,
    pub routes: Vec<RouteCheckSpec>,
    pub runtime: RuntimeDraft,
    pub transaction: TransactionSpec,
}

#[derive(Default)]
pub(crate) struct RuntimeDraft {
    pub start_command: Option<String>,
    pub base_url: Option<String>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AalImport {
    pub kind: String,
    pub id: String,
    pub version: Option<String>,
    pub line: usize,
}
