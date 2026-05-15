use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const TAIL_LIMIT: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSummary {
    pub stdout: String,
    pub stderr: String,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
}

pub fn paths(log_dir: &Path, prefix: &str) -> (PathBuf, PathBuf) {
    (
        log_dir.join(format!("{prefix}.stdout.log")),
        log_dir.join(format!("{prefix}.stderr.log")),
    )
}

pub fn from_files(stdout_path: &Path, stderr_path: &Path) -> Result<OutputSummary> {
    let stdout = tail_file(stdout_path)?;
    let stderr = tail_file(stderr_path)?;
    Ok(OutputSummary {
        stdout: stdout.tail.clone(),
        stderr: stderr.tail.clone(),
        stdout_path: Some(stdout_path.display().to_string()),
        stderr_path: Some(stderr_path.display().to_string()),
        stdout_tail: stdout.tail,
        stderr_tail: stderr.tail,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
        stdout_bytes: stdout.bytes,
        stderr_bytes: stderr.bytes,
    })
}

pub fn from_bytes(stdout: &[u8], stderr: &[u8]) -> OutputSummary {
    let stdout_tail = tail_bytes(stdout);
    let stderr_tail = tail_bytes(stderr);
    OutputSummary {
        stdout: stdout_tail.clone(),
        stderr: stderr_tail.clone(),
        stdout_path: None,
        stderr_path: None,
        stdout_tail,
        stderr_tail,
        stdout_truncated: stdout.len() > TAIL_LIMIT,
        stderr_truncated: stderr.len() > TAIL_LIMIT,
        stdout_bytes: stdout.len() as u64,
        stderr_bytes: stderr.len() as u64,
    }
}

fn tail_file(path: &Path) -> Result<Tail> {
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let bytes = metadata.len();
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let start = bytes.saturating_sub(TAIL_LIMIT as u64);
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(Tail {
        tail: String::from_utf8_lossy(&buffer).to_string(),
        truncated: start > 0,
        bytes,
    })
}

fn tail_bytes(value: &[u8]) -> String {
    let start = value.len().saturating_sub(TAIL_LIMIT);
    String::from_utf8_lossy(&value[start..]).to_string()
}

struct Tail {
    tail: String,
    truncated: bool,
    bytes: u64,
}
