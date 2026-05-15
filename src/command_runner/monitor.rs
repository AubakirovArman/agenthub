use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;

use super::cancel::{read_cancel_request, write_cancel_status, CancelStatus};
use super::process::terminate_process_tree;

pub struct WaitOutcome {
    pub timed_out: bool,
    pub cancelled_reason: Option<String>,
}

pub fn wait(
    child: &mut Child,
    started: Instant,
    timeout: Duration,
    tx_dir: Option<&Path>,
    node: &str,
    log_paths: Option<&(PathBuf, PathBuf)>,
) -> Result<WaitOutcome> {
    let mut last_heartbeat = Instant::now();
    let interval = heartbeat_interval();
    let mut last_output = Instant::now();
    let mut last_size = output_size(log_paths);

    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if let Some(reason) = cancel_reason(tx_dir)? {
            terminate_process_tree(child);
            write_cancelled(tx_dir, &reason)?;
            return Ok(WaitOutcome {
                timed_out: false,
                cancelled_reason: Some(reason),
            });
        }
        if started.elapsed() >= timeout {
            terminate_process_tree(child);
            return Ok(WaitOutcome {
                timed_out: true,
                cancelled_reason: None,
            });
        }
        let size = output_size(log_paths);
        if size > last_size {
            last_size = size;
            last_output = Instant::now();
        }
        if let Some(tx_dir) = tx_dir {
            if last_heartbeat.elapsed() >= interval {
                write_heartbeat(tx_dir, node, started.elapsed(), last_output.elapsed())?;
                last_heartbeat = Instant::now();
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    Ok(WaitOutcome {
        timed_out: false,
        cancelled_reason: None,
    })
}

fn cancel_reason(tx_dir: Option<&Path>) -> Result<Option<String>> {
    let Some(tx_dir) = tx_dir else {
        return Ok(None);
    };
    Ok(read_cancel_request(tx_dir)?.map(|request| request.reason))
}

fn write_cancelled(tx_dir: Option<&Path>, reason: &str) -> Result<()> {
    if let Some(tx_dir) = tx_dir {
        write_cancel_status(
            tx_dir,
            &CancelStatus {
                cancelled: true,
                reason: Some(reason.to_string()),
            },
        )?;
    }
    Ok(())
}

fn write_heartbeat(
    tx_dir: &Path,
    node: &str,
    elapsed: Duration,
    last_output: Duration,
) -> Result<()> {
    let path = tx_dir.join("heartbeat.jsonl");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let event = json!({
        "ts": Utc::now(),
        "event": "HEARTBEAT",
        "node": node,
        "elapsed_sec": elapsed.as_secs(),
        "last_output_sec": last_output.as_secs(),
    });
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{}", serde_json::to_string(&event)?)?;
    Ok(())
}

fn output_size(log_paths: Option<&(PathBuf, PathBuf)>) -> u64 {
    let Some((stdout, stderr)) = log_paths else {
        return 0;
    };
    file_len(stdout) + file_len(stderr)
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

fn heartbeat_interval() -> Duration {
    std::env::var("AGENTHUB_HEARTBEAT_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_secs(30))
}
