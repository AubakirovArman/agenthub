use std::collections::BTreeMap;

use chrono::Utc;

use crate::analytics::{AnalyticsBucket, AnalyticsRecord, AnalyticsSummary, AnalyticsTotals};

pub fn summarize(records: &[AnalyticsRecord]) -> AnalyticsSummary {
    let totals = totals(records);
    AnalyticsSummary {
        version: "analytics.summary.v1".to_string(),
        updated_at: Utc::now(),
        success_rate: rate(totals.success, totals.runs),
        rollback_rate: rate(totals.rollback, totals.runs),
        repair_rate: rate(totals.repair, totals.runs),
        human_block_rate: rate(totals.human_block, totals.runs),
        dangerous_diff_rate: rate(totals.dangerous_diff, totals.runs),
        average_time_to_commit_ms: average_commit_ms(records),
        by_task_type: buckets(records, |record| Some(record.task_type.as_str())),
        by_topology: buckets(records, |record| record.topology.as_deref()),
        by_model: buckets(records, |record| record.model.as_deref()),
        by_verifier: buckets(records, |record| record.verifier_profile.as_deref()),
        by_skill: skill_buckets(records),
        totals,
    }
}

fn totals(records: &[AnalyticsRecord]) -> AnalyticsTotals {
    AnalyticsTotals {
        runs: records.len(),
        success: records.iter().filter(|record| record.success).count(),
        rollback: records.iter().filter(|record| record.rollback).count(),
        repair: records.iter().filter(|record| record.repair).count(),
        human_block: records.iter().filter(|record| record.human_block).count(),
        dangerous_diff: records
            .iter()
            .filter(|record| record.dangerous_diff)
            .count(),
    }
}

fn buckets<F>(records: &[AnalyticsRecord], key: F) -> Vec<AnalyticsBucket>
where
    F: Fn(&AnalyticsRecord) -> Option<&str>,
{
    let mut buckets = BTreeMap::new();
    for record in records {
        if let Some(key) = key(record) {
            add(&mut buckets, key, record);
        }
    }
    buckets.into_values().collect()
}

fn skill_buckets(records: &[AnalyticsRecord]) -> Vec<AnalyticsBucket> {
    let mut buckets = BTreeMap::new();
    for record in records {
        for skill in &record.skills {
            add(&mut buckets, skill, record);
        }
    }
    buckets.into_values().collect()
}

fn add(buckets: &mut BTreeMap<String, AnalyticsBucket>, key: &str, record: &AnalyticsRecord) {
    let bucket = buckets
        .entry(key.to_string())
        .or_insert_with(|| AnalyticsBucket {
            key: key.to_string(),
            ..AnalyticsBucket::default()
        });
    bucket.runs += 1;
    bucket.success += usize::from(record.success);
    bucket.rollback += usize::from(record.rollback);
    bucket.repair += usize::from(record.repair);
    bucket.human_block += usize::from(record.human_block);
    bucket.total_cost_usd += record.cost_usd;
    bucket.total_latency_ms += record.duration_ms;
}

fn average_commit_ms(records: &[AnalyticsRecord]) -> f64 {
    let committed = records
        .iter()
        .filter(|record| record.success)
        .map(|record| record.duration_ms)
        .collect::<Vec<_>>();
    if committed.is_empty() {
        0.0
    } else {
        committed.iter().sum::<u64>() as f64 / committed.len() as f64
    }
}

fn rate(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}
