use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub struct SupervisedChild {
    child: Child,
}

pub fn run_shell(command: &str, cwd: &Path, timeout: Duration) -> Result<CommandResult> {
    let started = Instant::now();
    let mut process = Command::new("sh");
    process
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_process_group(&mut process);

    let mut child = process
        .spawn()
        .with_context(|| format!("spawn command `{command}` in {}", cwd.display()))?;

    let mut timed_out = false;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            terminate_process_tree(&mut child);
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let output = child
        .wait_with_output()
        .with_context(|| format!("wait for command `{command}`"))?;
    let exit_code = output.status.code();
    let success = output.status.success() && !timed_out;

    Ok(CommandResult {
        command: command.to_string(),
        cwd: cwd.display().to_string(),
        exit_code,
        success,
        timed_out,
        duration_ms: started.elapsed().as_millis(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

pub fn spawn_shell(command: &str, cwd: &Path) -> Result<SupervisedChild> {
    let mut process = Command::new("sh");
    process
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    configure_process_group(&mut process);

    let child = process
        .spawn()
        .with_context(|| format!("spawn command `{command}` in {}", cwd.display()))?;
    Ok(SupervisedChild { child })
}

impl SupervisedChild {
    pub fn terminate(&mut self) {
        terminate_process_tree(&mut self.child);
    }
}

impl Drop for SupervisedChild {
    fn drop(&mut self) {
        self.terminate();
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) {
    let pgid = -(child.id() as i32);
    unsafe {
        libc::kill(pgid, libc::SIGTERM);
    }

    let grace_started = Instant::now();
    while grace_started.elapsed() < Duration::from_secs(1) {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    unsafe {
        libc::kill(pgid, libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_timed_out_command() -> Result<()> {
        let result = run_shell("sleep 2", Path::new("."), Duration::from_millis(50))?;
        assert!(result.timed_out);
        assert!(!result.success);
        Ok(())
    }

    #[test]
    fn timeout_terminates_background_child() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let marker = dir.path().join("late-marker");
        let command = format!("(sleep 2; touch '{}') & wait", marker.display());

        let result = run_shell(&command, dir.path(), Duration::from_millis(50))?;
        thread::sleep(Duration::from_millis(300));

        assert!(result.timed_out);
        assert!(!marker.exists());
        Ok(())
    }
}
