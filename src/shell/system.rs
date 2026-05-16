use std::fs::File;
use std::io::{self, IsTerminal, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;

use crate::command_policy;
use crate::command_runner::{run_shell_with_sandbox_logged, CommandResult, CommandSandbox};
use crate::observability::redact_text;

use super::format;

pub(super) fn run(root: &Path, command: &str) -> Result<()> {
    if command.trim().is_empty() {
        format::error("shell command is empty");
        return Ok(());
    }
    let policy = command_policy::classify_shell_command(root, command)?;
    match policy.classification.as_str() {
        "restricted" => {
            format::error(&format!("blocked restricted command: {command}"));
            return Ok(());
        }
        "needs_approval" if !confirm(&format!("Approve command `{command}`?"), false)? => {
            println!("command skipped");
            return Ok(());
        }
        _ => {}
    }
    let logs = root.join(".agent/shell/commands");
    let prefix = format!("shell-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let stdout_log = logs.join(format!("{prefix}.stdout.log"));
    let stderr_log = logs.join(format!("{prefix}.stderr.log"));
    format::info("command running");
    println!("stdout_log {}", stdout_log.display());
    println!("stderr_log {}", stderr_log.display());
    println!("Hint: press Ctrl-C to return to prompt after the command exits; logs are live.");
    let result = run_with_live_tail(root, command, &logs, &prefix, &stdout_log, &stderr_log)?;
    print_result(result);
    Ok(())
}

fn run_with_live_tail(
    root: &Path,
    command: &str,
    logs: &Path,
    prefix: &str,
    stdout_log: &Path,
    stderr_log: &Path,
) -> Result<CommandResult> {
    let (tx, rx) = mpsc::channel();
    let command = command.to_string();
    let root = root.to_path_buf();
    let logs = logs.to_path_buf();
    let prefix = prefix.to_string();
    thread::spawn(move || {
        let result = run_shell_with_sandbox_logged(
            &command,
            &root,
            Duration::from_secs(900),
            CommandSandbox::default(),
            &logs,
            &prefix,
        );
        let _ = tx.send(result);
    });

    let started = Instant::now();
    let mut stdout_offset = 0;
    let mut stderr_offset = 0;
    let mut last_heartbeat = 0;
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(result) => {
                print_new_output(stdout_log, &mut stdout_offset, "stdout")?;
                print_new_output(stderr_log, &mut stderr_offset, "stderr")?;
                return result;
            }
            Err(RecvTimeoutError::Timeout) => {
                print_new_output(stdout_log, &mut stdout_offset, "stdout")?;
                print_new_output(stderr_log, &mut stderr_offset, "stderr")?;
                let elapsed = started.elapsed().as_secs();
                if elapsed >= last_heartbeat + 5 {
                    println!("running {elapsed}s");
                    last_heartbeat = elapsed;
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                anyhow::bail!("command runner stopped before returning a result");
            }
        }
    }
}

fn print_new_output(path: &Path, offset: &mut u64, label: &str) -> Result<()> {
    let Some(chunk) = read_new(path, offset)? else {
        return Ok(());
    };
    if chunk.trim().is_empty() {
        return Ok(());
    }
    let safe = redact_text(&chunk).unwrap_or(chunk);
    println!("{label}:");
    print!("{safe}");
    if !safe.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn read_new(path: &Path, offset: &mut u64) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    if len <= *offset {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(*offset))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    *offset = len;
    Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
}

fn print_result(result: CommandResult) {
    let status = if result.success {
        "completed"
    } else {
        "failed"
    };
    if result.success {
        format::success(&format!("command {status} in {} ms", result.duration_ms));
    } else {
        format::error(&format!("command {status} in {} ms", result.duration_ms));
    }
    if !result.stdout.trim().is_empty() {
        println!("stdout:\n{}", result.stdout.trim());
    }
    if !result.stderr.trim().is_empty() {
        println!("stderr:\n{}", result.stderr.trim());
    }
    if let Some(path) = result.stdout_path {
        println!("stdout_log {path}");
    }
    if let Some(path) = result.stderr_path {
        println!("stderr_log {path}");
    }
}

fn confirm(question: &str, default_yes: bool) -> Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(default_yes);
    }
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{question} {suffix} ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let answer = line.trim().to_ascii_lowercase();
    Ok(match answer.as_str() {
        "" => default_yes,
        "y" | "yes" | "д" | "да" => true,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::read_new;

    #[test]
    fn live_tail_reads_only_new_bytes() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("command.log");
        fs::write(&path, "one\n")?;
        let mut offset = 0;

        assert_eq!(read_new(&path, &mut offset)?.as_deref(), Some("one\n"));
        assert_eq!(read_new(&path, &mut offset)?, None);

        fs::write(&path, "one\ntwo\n")?;
        assert_eq!(read_new(&path, &mut offset)?.as_deref(), Some("two\n"));
        Ok(())
    }
}
