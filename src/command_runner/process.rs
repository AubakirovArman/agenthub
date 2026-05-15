use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
pub(super) fn configure_process_group(command: &mut Command) {
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
pub(super) fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
pub(super) fn terminate_process_tree(child: &mut Child) {
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
pub(super) fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
}

pub fn process_control_label() -> &'static str {
    if cfg!(unix) {
        "unix_process_group_sigterm_sigkill"
    } else if cfg!(windows) {
        "windows_child_kill_fallback"
    } else {
        "child_kill_fallback"
    }
}
