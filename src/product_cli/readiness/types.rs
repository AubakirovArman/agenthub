use std::{collections::BTreeSet, path::PathBuf};

use serde::Serialize;

pub(super) const OBJECTIVE: &str = "API-native 1.0 bridge with DeepSeek/Kimi, Chat/Ops/Project, memory, observability, RC dogfood evidence, and post-1.0 roadmap sequencing";

#[derive(Debug, Clone, Copy)]
pub struct AuditOptions {
    pub json: bool,
    pub no_refresh: bool,
}

#[derive(Debug)]
pub struct AuditRenderResult {
    pub output: String,
    pub failed: bool,
}

#[derive(Debug, Serialize)]
pub struct ReadinessAuditReport {
    pub objective: String,
    pub status: String,
    pub failed: bool,
    pub sources: ReadinessSources,
    pub evidence: String,
    pub dogfood_history: String,
    pub kimi_auth_report: String,
    pub metrics: ReadinessMetrics,
    pub checks: Vec<ReadinessCheck>,
    pub next: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadinessBlockerReport {
    pub objective: String,
    pub status: String,
    pub failed: bool,
    pub sources: ReadinessSources,
    pub evidence: String,
    pub dogfood_history: String,
    pub kimi_auth_report: String,
    pub metrics: ReadinessMetrics,
    pub blockers: Vec<ReadinessBlocker>,
    pub next: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadinessSources {
    pub api_native_plan: String,
    pub post_1_0_plan: String,
    pub repo_roadmap: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ReadinessMetrics {
    pub real_sessions: usize,
    pub required_sessions: usize,
    pub ops_flows: usize,
    pub required_ops_flows: usize,
    pub project_edit_flows: usize,
    pub required_project_edit_flows: usize,
    pub cost_receipts: usize,
    pub required_cost_receipts: usize,
    pub open_blockers: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadinessCheck {
    pub id: String,
    pub status: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker_kind: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_commands: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadinessBlocker {
    pub id: String,
    pub status: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker_kind: Option<String>,
    pub next_commands: Vec<String>,
}

#[derive(Default)]
pub(super) struct EvidenceSummary {
    pub real_sessions: usize,
    pub ops_flows: usize,
    pub project_edit_flows: usize,
    pub cost_receipts: usize,
    pub providers_passed: BTreeSet<String>,
    pub checks_passed: BTreeSet<String>,
    pub open_blockers: usize,
    pub open_blocker_ids: BTreeSet<String>,
}

pub(super) struct AuditConfig {
    pub evidence: PathBuf,
    pub history_dir: PathBuf,
    pub kimi_report: PathBuf,
    pub v04_plan: PathBuf,
    pub after_plan: PathBuf,
    pub roadmap_doc: PathBuf,
    pub required_providers: Vec<String>,
    pub required_checks: Vec<String>,
    pub min_sessions: usize,
    pub min_ops: usize,
    pub min_project: usize,
    pub min_cost: usize,
}

impl AuditConfig {
    pub(super) fn from_env(project_root: &std::path::Path) -> Self {
        let min_sessions = env_usize(
            &[
                "AGENTHUB_API_AUDIT_MIN_REAL_SESSIONS",
                "AGENTHUB_RC_MIN_REAL_SESSIONS",
            ],
            100,
        );
        let min_cost = env_usize(
            &[
                "AGENTHUB_API_AUDIT_MIN_COST_RECEIPTS",
                "AGENTHUB_RC_MIN_COST_RECEIPTS",
            ],
            min_sessions,
        );
        Self {
            evidence: env_path(
                &["AGENTHUB_API_AUDIT_EVIDENCE", "AGENTHUB_RC_EVIDENCE"],
                project_root.join("target/dogfood/rc-evidence.jsonl"),
            ),
            history_dir: env_path(
                &[
                    "AGENTHUB_API_AUDIT_HISTORY_DIR",
                    "AGENTHUB_DOGFOOD_HISTORY_DIR",
                ],
                project_root.join("target/dogfood/history"),
            ),
            kimi_report: env_path(
                &[
                    "AGENTHUB_API_AUDIT_KIMI_REPORT",
                    "AGENTHUB_RC_KIMI_AUTH_REPORT",
                    "AGENTHUB_KIMI_AUTH_REPORT",
                ],
                project_root.join("target/dogfood/kimi-auth-report.json"),
            ),
            v04_plan: env_path(
                &["AGENTHUB_API_AUDIT_V04_PLAN"],
                PathBuf::from(
                    "/mnt/hf_model_weights/arman/3bit/agenthub_v04_api_native/agenthub_v04_api_native.md",
                ),
            ),
            after_plan: env_path(
                &["AGENTHUB_API_AUDIT_AFTER_PLAN"],
                PathBuf::from("/mnt/hf_model_weights/arman/3bit/agenthub_after_10_roadmap.md"),
            ),
            roadmap_doc: env_path(
                &["AGENTHUB_API_AUDIT_ROADMAP_DOC"],
                project_root.join("docs/roadmap-after-1.0.ru.md"),
            ),
            required_providers: env_csv(
                &[
                    "AGENTHUB_API_AUDIT_REQUIRED_PROVIDERS",
                    "AGENTHUB_RC_REQUIRED_PROVIDERS",
                ],
                "deepseek,kimi",
            ),
            required_checks: env_csv(
                &[
                    "AGENTHUB_API_AUDIT_REQUIRED_CHECKS",
                    "AGENTHUB_RC_REQUIRED_CHECKS",
                ],
                "chat_no_bootstrap,ops_no_bootstrap,resume,rewind,stats,cost_receipts,ops_receipts,approval_ux,long_session_latency",
            ),
            min_sessions,
            min_ops: env_usize(
                &[
                    "AGENTHUB_API_AUDIT_MIN_OPS_FLOWS",
                    "AGENTHUB_RC_MIN_OPS_FLOWS",
                ],
                20,
            ),
            min_project: env_usize(
                &[
                    "AGENTHUB_API_AUDIT_MIN_PROJECT_EDIT_FLOWS",
                    "AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS",
                ],
                20,
            ),
            min_cost,
        }
    }
}

pub(super) fn next_commands() -> Vec<String> {
    [
        "agenthub providers recovery --json",
        "agenthub providers inspect-key kimi",
        "agenthub providers inspect-key kimi --from-file <new-key-file>",
        "agenthub providers preflight-key kimi --from-file <new-key-file>",
        "agenthub providers rc-unblock kimi --from-file <new-key-file>",
        "agenthub providers unblock kimi",
        "agenthub providers rotate-key kimi --from-file <new-key-file>",
        "scripts/kimi-key-rotate.sh --from-file <new-key-file>",
        "agenthub providers rc-unblock kimi",
        "scripts/kimi-rc-unblock.sh",
        "agenthub providers test kimi",
        "scripts/kimi-auth-check.sh",
        "AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh",
        "agenthub readiness blockers --json --check",
        "agenthub readiness audit --json --check",
        "scripts/rc-evidence-collect.sh",
        "scripts/rc-dogfood-gate.sh --check",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub(super) fn env_usize(keys: &[&str], default: usize) -> usize {
    keys.iter()
        .find_map(|key| std::env::var(key).ok())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_path(keys: &[&str], default: PathBuf) -> PathBuf {
    keys.iter()
        .find_map(|key| std::env::var_os(key).filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .unwrap_or(default)
}

fn env_csv(keys: &[&str], default: &str) -> Vec<String> {
    keys.iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| default.to_string())
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
