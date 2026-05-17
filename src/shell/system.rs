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
use crate::home;
use crate::observability::redact_text;
use crate::ops;
use crate::tool_permissions::{self, ToolPermissionDecision};

use super::format;

pub(super) fn run(root: &Path, command: &str) -> Result<Option<CommandResult>> {
    if command.trim().is_empty() {
        format::error("shell command is empty");
        return Ok(None);
    }
    let permission = permission_decision(command);
    print_permission(&permission);
    print_ops_context(command, &permission);
    let policy = command_policy::classify_shell_command(root, command)?;
    if policy.classification == "restricted" {
        format::error(&format!("blocked restricted command: {command}"));
        return Ok(None);
    }
    let policy_needs_approval = policy.classification == "needs_approval";
    let untrusted_ops_host = permission.profile == tool_permissions::ToolPermissionProfile::OpsHost
        && ops::command_trust(command).unwrap_or(ops::OpsHostTrust::Unknown)
            == ops::OpsHostTrust::Untrusted;
    if permission.approval_required || policy_needs_approval || untrusted_ops_host {
        let reason = if permission.approval_required {
            permission.reason.as_str()
        } else if untrusted_ops_host {
            "Ops target is marked untrusted"
        } else {
            policy
                .matched_policy
                .as_deref()
                .unwrap_or("matched command policy")
        };
        if !confirm(&format!("Approve command `{command}` ({reason})?"), false)? {
            println!("command skipped");
            return Ok(None);
        }
    }
    let logs = if home::project_has_shell_state(root) {
        root.join(".agent/shell/commands")
    } else {
        home::global_shell_commands_dir(root)
    };
    let prefix = format!("shell-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let stdout_log = logs.join(format!("{prefix}.stdout.log"));
    let stderr_log = logs.join(format!("{prefix}.stderr.log"));
    format::info("command running");
    println!("stdout_log {}", stdout_log.display());
    println!("stderr_log {}", stderr_log.display());
    println!("Hint: press Ctrl-C to return to prompt after the command exits; logs are live.");
    let result = run_with_live_tail(root, command, &logs, &prefix, &stdout_log, &stderr_log)?;
    print_result(result.clone());
    Ok(Some(result))
}

pub(super) fn permission_decision(command: &str) -> ToolPermissionDecision {
    tool_permissions::classify_shell_command(command)
}

fn print_permission(decision: &ToolPermissionDecision) {
    println!(
        "tool_permission tool={} profile={} risk={} approval_required={}",
        decision.tool,
        decision.profile.as_str(),
        decision.risk.as_str(),
        decision.approval_required
    );
    println!("reason {}", decision.reason);
}

fn print_ops_context(command: &str, decision: &ToolPermissionDecision) {
    if decision.profile != tool_permissions::ToolPermissionProfile::OpsHost {
        return;
    }
    println!("ops_target {}", ops::command_target(command));
    let trust = ops::command_trust(command).unwrap_or(ops::OpsHostTrust::Unknown);
    println!("ops_trust {}", trust.as_str());
    println!("ops_receipt enabled");
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
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
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

    use crate::tool_permissions::{ToolPermissionProfile, ToolRisk};

    use super::{permission_decision, read_new, run_with_live_tail};

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

    #[test]
    fn ops_command_without_project_runtime_does_not_create_agent_dir() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let logs = tempfile::tempdir()?;
        let prefix = "shell-test";
        let stdout_log = logs.path().join(format!("{prefix}.stdout.log"));
        let stderr_log = logs.path().join(format!("{prefix}.stderr.log"));

        let result = run_with_live_tail(
            dir.path(),
            "printf ops-ok",
            logs.path(),
            prefix,
            &stdout_log,
            &stderr_log,
        )?;

        assert!(result.success);
        assert_eq!(result.stdout, "ops-ok");
        assert!(!dir.path().join(".agent").exists());
        Ok(())
    }

    #[test]
    fn permission_decision_marks_risky_ops_commands() {
        let decision = permission_decision("kubectl delete pod api-1");

        assert_eq!(decision.profile, ToolPermissionProfile::OpsHost);
        assert_eq!(decision.risk, ToolRisk::High);
        assert!(decision.approval_required);
        assert!(decision.reason.contains("mutate"));
    }
}
