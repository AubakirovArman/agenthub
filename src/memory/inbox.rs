use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::storage::append_jsonl;
use super::{memory_paths, new_memory_id, write_typed_fact, TypedMemoryInput};

const INBOX_FILE: &str = "inbox.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInboxItem {
    pub id: String,
    pub status: String,
    pub kind: String,
    pub domain: String,
    pub content: Value,
    pub source: String,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub memory_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInboxInput {
    pub kind: String,
    pub domain: String,
    pub content: Value,
    pub source: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInboxReview {
    pub total: usize,
    pub pending: usize,
    pub reviewed: usize,
    pub groups: Vec<MemoryInboxGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInboxGroup {
    pub key: String,
    pub domain: String,
    pub kind: String,
    pub confidence_band: String,
    pub pending: usize,
    pub reviewed: usize,
    pub duplicate_or_conflict: bool,
    pub items: Vec<MemoryInboxReviewItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInboxReviewItem {
    pub id: String,
    pub status: String,
    pub kind: String,
    pub domain: String,
    pub source: String,
    pub summary: String,
    pub confidence: Option<f64>,
    pub confidence_band: String,
    pub group_key: String,
    pub promotion_diff: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub memory_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxDecision {
    Approve,
    Reject,
}

pub fn add_inbox_candidate(root: &Path, input: MemoryInboxInput) -> Result<MemoryInboxItem> {
    let paths = memory_paths(root)?;
    let item = MemoryInboxItem {
        id: new_memory_id("inbox"),
        status: "pending".to_string(),
        kind: input.kind,
        domain: input.domain,
        content: input.content,
        source: input.source,
        reason: input.reason,
        created_at: Utc::now(),
        reviewed_at: None,
        memory_id: None,
    };
    append_jsonl(&paths.memory.join(INBOX_FILE), &item)?;
    Ok(item)
}

pub fn list_inbox(root: &Path, include_reviewed: bool) -> Result<Vec<MemoryInboxItem>> {
    let paths = memory_paths(root)?;
    let mut latest = BTreeMap::<String, MemoryInboxItem>::new();
    for item in read_inbox_items(&paths.memory.join(INBOX_FILE))? {
        latest.insert(item.id.clone(), item);
    }
    let mut items = latest.into_values().collect::<Vec<_>>();
    if !include_reviewed {
        items.retain(|item| item.status == "pending");
    }
    items.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(items)
}

pub fn review_inbox_view(root: &Path, include_reviewed: bool) -> Result<MemoryInboxReview> {
    let items = list_inbox(root, include_reviewed)?;
    let mut groups = BTreeMap::<String, MemoryInboxGroup>::new();
    for item in items {
        let view = review_item(item);
        let entry = groups
            .entry(view.group_key.clone())
            .or_insert_with(|| MemoryInboxGroup {
                key: view.group_key.clone(),
                domain: view.domain.clone(),
                kind: view.kind.clone(),
                confidence_band: view.confidence_band.clone(),
                pending: 0,
                reviewed: 0,
                duplicate_or_conflict: false,
                items: Vec::new(),
            });
        if view.status == "pending" {
            entry.pending += 1;
        } else {
            entry.reviewed += 1;
        }
        entry.items.push(view);
    }

    let mut groups = groups.into_values().collect::<Vec<_>>();
    for group in &mut groups {
        group.duplicate_or_conflict = group.items.len() > 1;
        group.items.sort_by(|a, b| {
            rank_key(b)
                .cmp(&rank_key(a))
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        group.confidence_band = group
            .items
            .first()
            .map(|item| item.confidence_band.clone())
            .unwrap_or_else(|| "unknown".to_string());
    }
    groups.sort_by(|a, b| {
        group_rank_key(b)
            .cmp(&group_rank_key(a))
            .then_with(|| a.key.cmp(&b.key))
    });

    let pending = groups.iter().map(|group| group.pending).sum();
    let reviewed = groups.iter().map(|group| group.reviewed).sum();
    Ok(MemoryInboxReview {
        total: pending + reviewed,
        pending,
        reviewed,
        groups,
    })
}

pub fn review_inbox(root: &Path, id: &str, decision: InboxDecision) -> Result<MemoryInboxItem> {
    let paths = memory_paths(root)?;
    let Some(mut item) = list_inbox(root, true)?
        .into_iter()
        .find(|item| item.id == id)
    else {
        return Err(anyhow!("memory inbox item `{id}` not found"));
    };
    if item.status != "pending" {
        return Err(anyhow!(
            "memory inbox item `{id}` is already {}",
            item.status
        ));
    }

    item.status = match decision {
        InboxDecision::Approve => "approved".to_string(),
        InboxDecision::Reject => "rejected".to_string(),
    };
    item.reviewed_at = Some(Utc::now());

    if decision == InboxDecision::Approve {
        let record = write_typed_fact(
            root,
            TypedMemoryInput {
                kind: item.kind.clone(),
                domain: item.domain.clone(),
                content: item.content.clone(),
                task_id: Some("memory_inbox_approved".to_string()),
                supersedes: None,
                confidence: Some(0.75),
                ttl_days: None,
                pinned: false,
                conflict_key: None,
            },
        )?;
        item.memory_id = Some(record.id);
    }

    append_jsonl(&paths.memory.join(INBOX_FILE), &item)?;
    Ok(item)
}

pub fn review_inbox_many(
    root: &Path,
    ids: &[String],
    decision: InboxDecision,
) -> Result<Vec<MemoryInboxItem>> {
    let mut seen = BTreeSet::new();
    let ids = ids
        .iter()
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .filter(|id| seen.insert((*id).to_string()))
        .collect::<Vec<_>>();
    if ids.is_empty() {
        return Err(anyhow!("memory inbox id is required"));
    }

    let items = list_inbox(root, true)?;
    let by_id = items
        .iter()
        .map(|item| (item.id.as_str(), item.status.as_str()))
        .collect::<BTreeMap<_, _>>();
    for id in &ids {
        let Some(status) = by_id.get(id) else {
            return Err(anyhow!("memory inbox item `{id}` not found"));
        };
        if *status != "pending" {
            return Err(anyhow!("memory inbox item `{id}` is already {status}"));
        }
    }

    ids.into_iter()
        .map(|id| review_inbox(root, id, decision))
        .collect()
}

fn review_item(item: MemoryInboxItem) -> MemoryInboxReviewItem {
    let summary = inbox_summary(&item.content);
    let confidence = inbox_confidence(&item.content);
    let confidence_band = confidence_band(confidence).to_string();
    let group_key = group_key(&item.domain, &item.kind, &summary);
    let promotion_diff = promotion_diff(&item);
    MemoryInboxReviewItem {
        id: item.id,
        status: item.status,
        kind: item.kind,
        domain: item.domain,
        source: item.source,
        summary,
        confidence,
        confidence_band,
        group_key,
        promotion_diff,
        created_at: item.created_at,
        reviewed_at: item.reviewed_at,
        memory_id: item.memory_id,
    }
}

fn inbox_summary(content: &Value) -> String {
    content
        .get("summary")
        .and_then(Value::as_str)
        .or_else(|| content.get("note").and_then(Value::as_str))
        .or_else(|| {
            content
                .get("evidence")
                .and_then(|value| value.get("request_excerpt"))
                .and_then(Value::as_str)
        })
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn inbox_confidence(content: &Value) -> Option<f64> {
    content.get("confidence").and_then(Value::as_f64)
}

fn confidence_band(confidence: Option<f64>) -> &'static str {
    match confidence {
        Some(value) if value >= 0.75 => "high",
        Some(value) if value >= 0.5 => "medium",
        Some(_) => "low",
        None => "unknown",
    }
}

fn group_key(domain: &str, kind: &str, summary: &str) -> String {
    let normalized = summary
        .chars()
        .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .take(12)
        .collect::<Vec<_>>()
        .join("-");
    format!(
        "{domain}:{kind}:{}",
        if normalized.is_empty() {
            "unspecified"
        } else {
            normalized.as_str()
        }
    )
}

fn promotion_diff(item: &MemoryInboxItem) -> String {
    match item.status.as_str() {
        "pending" => format!("pending -> committed {}/{}", item.domain, item.kind),
        "approved" => format!(
            "approved -> {}",
            item.memory_id.as_deref().unwrap_or("committed memory")
        ),
        "rejected" => "rejected -> no promotion".to_string(),
        other => format!("{other} -> no pending action"),
    }
}

fn rank_key(item: &MemoryInboxReviewItem) -> (u8, i64, String) {
    (
        band_rank(&item.confidence_band),
        item.confidence
            .map(|value| (value * 1_000.0) as i64)
            .unwrap_or_default(),
        item.id.clone(),
    )
}

fn group_rank_key(group: &MemoryInboxGroup) -> (u8, usize, String) {
    (
        band_rank(&group.confidence_band),
        group.pending,
        group.key.clone(),
    )
}

fn band_rank(band: &str) -> u8 {
    match band {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn read_inbox_items(path: &Path) -> Result<Vec<MemoryInboxItem>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut items = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        items.push(
            serde_json::from_str::<MemoryInboxItem>(&line)
                .with_context(|| format!("parse memory inbox item in {}", path.display()))?,
        );
    }
    Ok(items)
}
