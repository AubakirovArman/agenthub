use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::command_runner::CommandResult;
use crate::home;
use crate::memory::{self, TypedMemoryInput};
use crate::observability::{redact_text, sha256_short, write_jsonl};
use crate::tool_permissions::{ToolPermissionDecision, ToolPermissionProfile};

const HOSTS_FILE: &str = "hosts.jsonl";
const RECEIPTS_FILE: &str = "command_receipts.jsonl";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpsHostTrust {
    Trusted,
    Unknown,
    Untrusted,
}

impl OpsHostTrust {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "trusted" | "trust" => Ok(Self::Trusted),
            "unknown" | "" => Ok(Self::Unknown),
            "untrusted" | "deny" => Ok(Self::Untrusted),
            other => Err(anyhow!(
                "unsupported host trust `{other}`; expected trusted, unknown, or untrusted"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Unknown => "unknown",
            Self::Untrusted => "untrusted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsHostProfile {
    pub id: String,
    pub target: String,
    pub alias: Option<String>,
    pub trust: OpsHostTrust,
    pub note: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub command_count: usize,
}

#[derive(Debug, Clone)]
pub struct OpsHostInput {
    pub target: String,
    pub alias: Option<String>,
    pub trust: OpsHostTrust,
    pub note: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsRunbookCard {
    pub id: String,
    pub memory_id: String,
    pub title: String,
    pub host: Option<String>,
    pub command: Option<String>,
    pub summary: String,
    pub confidence: Option<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OpsRunbookInput {
    pub title: String,
    pub host: Option<String>,
    pub command: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpsCommandReceipt {
    pub id: String,
    pub host_id: String,
    pub target: String,
    pub trust: OpsHostTrust,
    pub command: String,
    pub profile: String,
    pub risk: String,
    pub approval_required: bool,
    pub reason: String,
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
    pub timed_out: Option<bool>,
    pub duration_ms: Option<u128>,
    pub stdout_log: Option<String>,
    pub stderr_log: Option<String>,
    pub stdout_tail: Option<String>,
    pub stderr_tail: Option<String>,
    pub runbook_cards: Vec<String>,
    pub created_at: DateTime<Utc>,
}

pub fn upsert_host(_root: &Path, input: OpsHostInput) -> Result<OpsHostProfile> {
    let target = canonical_target(&input.target);
    if target.is_empty() {
        return Err(anyhow!("ops host target is required"));
    }
    let now = Utc::now();
    let existing = find_host_by_target(&target)?;
    let profile = OpsHostProfile {
        id: host_id(&target),
        target,
        alias: input
            .alias
            .or_else(|| existing.as_ref().and_then(|item| item.alias.clone())),
        trust: input.trust,
        note: input
            .note
            .or_else(|| existing.as_ref().and_then(|item| item.note.clone())),
        source: input.source,
        created_at: existing.as_ref().map(|item| item.created_at).unwrap_or(now),
        updated_at: now,
        last_seen_at: existing.as_ref().and_then(|item| item.last_seen_at),
        command_count: existing
            .as_ref()
            .map(|item| item.command_count)
            .unwrap_or(0),
    };
    append_host(&profile)?;
    Ok(profile)
}

pub fn list_hosts(_root: &Path) -> Result<Vec<OpsHostProfile>> {
    let mut latest = BTreeMap::<String, OpsHostProfile>::new();
    for host in read_hosts()? {
        latest.insert(host.id.clone(), host);
    }
    let mut hosts = latest.into_values().collect::<Vec<_>>();
    hosts.sort_by(|a, b| {
        a.target
            .cmp(&b.target)
            .then_with(|| a.alias.cmp(&b.alias))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(hosts)
}

pub fn command_trust(command: &str) -> Result<OpsHostTrust> {
    let target = command_target(command);
    Ok(find_host_by_target(&target)?
        .map(|host| host.trust)
        .unwrap_or(OpsHostTrust::Unknown))
}

pub fn add_runbook_card(root: &Path, input: OpsRunbookInput) -> Result<OpsRunbookCard> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(anyhow!("ops runbook title is required"));
    }
    let record = memory::write_typed_fact(
        root,
        TypedMemoryInput {
            kind: "runbook_step".to_string(),
            domain: "ops".to_string(),
            content: json!({
                "summary": title,
                "host": input.host.as_deref().unwrap_or(""),
                "command": input.command.as_deref().unwrap_or(""),
                "note": input.note.as_deref().unwrap_or(""),
            }),
            task_id: Some("ops_runbook_card".to_string()),
            supersedes: None,
            confidence: Some(0.82),
            ttl_days: None,
            pinned: true,
            conflict_key: input
                .host
                .as_deref()
                .map(|host| format!("ops_runbook:{}:{title}", canonical_target(host))),
        },
    )?;
    Ok(runbook_card_from_record(&record))
}

pub fn list_runbook_cards(root: &Path, host: Option<&str>) -> Result<Vec<OpsRunbookCard>> {
    let target = host.map(canonical_target).filter(|value| !value.is_empty());
    let mut cards = memory::retrieve_relevant(root, "ops", 100)?
        .into_iter()
        .filter(|record| record.kind == "runbook_step")
        .map(|record| runbook_card_from_record(&record))
        .filter(|card| {
            target.as_deref().is_none_or(|host| {
                card.host.as_deref().map(canonical_target).as_deref() == Some(host)
            })
        })
        .collect::<Vec<_>>();
    cards.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.title.cmp(&b.title))
    });
    Ok(cards)
}

pub fn list_receipts(
    _root: &Path,
    limit: usize,
    host: Option<&str>,
) -> Result<Vec<OpsCommandReceipt>> {
    let target = host.map(canonical_target).filter(|value| !value.is_empty());
    let mut receipts = read_receipts(&ops_dir().join(RECEIPTS_FILE))?;
    if let Some(target) = target {
        receipts.retain(|receipt| canonical_target(&receipt.target) == target);
    }
    receipts.sort_by_key(|receipt| Reverse(receipt.created_at));
    receipts.truncate(limit);
    Ok(receipts)
}

pub fn record_command(
    root: &Path,
    command: &str,
    decision: &ToolPermissionDecision,
    result: Option<&CommandResult>,
) -> Result<Option<OpsCommandReceipt>> {
    if decision.profile != ToolPermissionProfile::OpsHost {
        return Ok(None);
    }
    let target = command_target(command);
    let profile = touch_host(root, &target)?;
    let runbook_cards = list_runbook_cards(root, Some(&profile.target))?
        .into_iter()
        .take(5)
        .map(|card| card.id)
        .collect::<Vec<_>>();
    let receipt = OpsCommandReceipt {
        id: format!("ops-cmd-{}", &Uuid::new_v4().to_string()[..8]),
        host_id: profile.id.clone(),
        target: profile.target.clone(),
        trust: profile.trust,
        command: redact_text(command).unwrap_or_else(|_| command.to_string()),
        profile: decision.profile.as_str().to_string(),
        risk: decision.risk.as_str().to_string(),
        approval_required: decision.approval_required,
        reason: decision.reason.clone(),
        success: result.map(|item| item.success),
        exit_code: result.and_then(|item| item.exit_code),
        timed_out: result.map(|item| item.timed_out),
        duration_ms: result.map(|item| item.duration_ms),
        stdout_log: result.and_then(|item| item.stdout_path.clone()),
        stderr_log: result.and_then(|item| item.stderr_path.clone()),
        stdout_tail: result.map(|item| redact_text(&item.stdout_tail).unwrap_or_default()),
        stderr_tail: result.map(|item| redact_text(&item.stderr_tail).unwrap_or_default()),
        runbook_cards,
        created_at: Utc::now(),
    };
    append_receipt(&receipt)?;
    Ok(Some(receipt))
}

pub fn command_target(command: &str) -> String {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    let Some(first) = tokens.first().map(|value| value.to_ascii_lowercase()) else {
        return "localhost".to_string();
    };
    match first.as_str() {
        "ssh" => ssh_target(&tokens).unwrap_or_else(|| "unknown-ssh-host".to_string()),
        "scp" | "rsync" => {
            file_transfer_target(&tokens).unwrap_or_else(|| "unknown-remote".to_string())
        }
        "systemctl" | "service" | "journalctl" | "docker" => "localhost".to_string(),
        "kubectl" | "helm" => "kubernetes-context".to_string(),
        "terraform" => "terraform-workspace".to_string(),
        _ => "localhost".to_string(),
    }
}

fn touch_host(root: &Path, target: &str) -> Result<OpsHostProfile> {
    let target = canonical_target(target);
    let now = Utc::now();
    let existing = find_host_by_target(&target)?;
    let was_new = existing.is_none();
    let profile = if let Some(mut host) = existing {
        host.last_seen_at = Some(now);
        host.updated_at = now;
        host.command_count += 1;
        host
    } else {
        OpsHostProfile {
            id: host_id(&target),
            target,
            alias: None,
            trust: OpsHostTrust::Unknown,
            note: Some("auto-detected from Ops command".to_string()),
            source: "auto".to_string(),
            created_at: now,
            updated_at: now,
            last_seen_at: Some(now),
            command_count: 1,
        }
    };
    append_host(&profile)?;
    if was_new {
        let _ = memory::write_typed_fact(
            root,
            TypedMemoryInput {
                kind: "host_profile".to_string(),
                domain: "ops".to_string(),
                content: json!({
                    "host_id": profile.id.as_str(),
                    "target": profile.target.as_str(),
                    "trust": profile.trust.as_str(),
                    "source": profile.source.as_str(),
                }),
                task_id: Some("ops_host_profile".to_string()),
                supersedes: None,
                confidence: Some(0.55),
                ttl_days: None,
                pinned: false,
                conflict_key: Some(format!("ops_host:{}", profile.id)),
            },
        );
    }
    Ok(profile)
}

fn append_host(host: &OpsHostProfile) -> Result<()> {
    write_jsonl(&ops_dir().join(HOSTS_FILE), &serde_json::to_value(host)?)
}

fn append_receipt(receipt: &OpsCommandReceipt) -> Result<()> {
    let value = serde_json::to_value(receipt)?;
    write_jsonl(&ops_dir().join(RECEIPTS_FILE), &value)?;
    write_jsonl(
        &ops_dir()
            .join("hosts")
            .join(&receipt.host_id)
            .join(RECEIPTS_FILE),
        &value,
    )
}

fn find_host_by_target(target: &str) -> Result<Option<OpsHostProfile>> {
    let id = host_id(target);
    Ok(read_hosts()?.into_iter().rev().find(|host| host.id == id))
}

fn read_hosts() -> Result<Vec<OpsHostProfile>> {
    read_jsonl(&ops_dir().join(HOSTS_FILE))
}

fn read_receipts(path: &Path) -> Result<Vec<OpsCommandReceipt>> {
    read_jsonl(path)
}

fn read_jsonl<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut rows = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        rows.push(
            serde_json::from_str(&line).with_context(|| format!("parse {}", path.display()))?,
        );
    }
    Ok(rows)
}

fn runbook_card_from_record(record: &memory::MemoryRecord) -> OpsRunbookCard {
    let summary = content_text(&record.content, "summary")
        .or_else(|| content_text(&record.content, "note"))
        .or_else(|| {
            record
                .content
                .get("evidence")
                .and_then(|value| value.get("request_excerpt"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Ops runbook step".to_string());
    let command = content_text(&record.content, "command").filter(|value| !value.is_empty());
    let host = content_text(&record.content, "host")
        .or_else(|| content_text(&record.content, "target"))
        .filter(|value| !value.is_empty());
    OpsRunbookCard {
        id: format!("runbook-{}", &record.id),
        memory_id: record.id.clone(),
        title: truncate_words(&summary, 10),
        host,
        command,
        summary,
        confidence: record.confidence,
        created_at: record.created_at,
    }
}

fn content_text(content: &Value, key: &str) -> Option<String> {
    content.get(key).and_then(Value::as_str).map(str::to_string)
}

fn truncate_words(value: &str, max_words: usize) -> String {
    let words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() <= max_words {
        return value.to_string();
    }
    format!("{}...", words[..max_words].join(" "))
}

fn ssh_target(tokens: &[&str]) -> Option<String> {
    let mut index = 1;
    while index < tokens.len() {
        let token = tokens[index];
        if matches!(token, "-p" | "-i" | "-F" | "-l" | "-o") {
            index += 2;
            continue;
        }
        if token.starts_with('-') {
            index += 1;
            continue;
        }
        return Some(canonical_target(token));
    }
    None
}

fn file_transfer_target(tokens: &[&str]) -> Option<String> {
    tokens.iter().skip(1).find_map(|token| {
        let token = token.trim_matches('"').trim_matches('\'');
        token
            .split_once(':')
            .map(|(target, _)| target)
            .filter(|target| !target.is_empty() && !target.starts_with('/'))
            .map(canonical_target)
    })
}

fn canonical_target(target: &str) -> String {
    target
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches(':')
        .to_ascii_lowercase()
}

fn host_id(target: &str) -> String {
    let normalized = canonical_target(target);
    let slug = normalized
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(40)
        .collect::<String>();
    let slug = if slug.is_empty() {
        "host"
    } else {
        slug.as_str()
    };
    format!("ops-host-{slug}-{}", sha256_short(normalized.as_bytes()))
}

fn ops_dir() -> PathBuf {
    home::base_dir().join("ops")
}

#[cfg(test)]
mod tests;
