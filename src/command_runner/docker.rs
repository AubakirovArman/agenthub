use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

use super::RemoteRunner;

pub(super) fn command(
    shell_command: &str,
    cwd: &Path,
    sandbox_level: u8,
    runner: &RemoteRunner,
) -> Result<Command> {
    let image = runner
        .endpoint
        .strip_prefix("docker://")
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("invalid docker runner endpoint `{}`", runner.endpoint))?;
    let mut process = Command::new("docker");
    process
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .args(["-v", &format!("{}:/workspace", cwd.display())])
        .args(["-w", "/workspace"])
        .args(["-e", &format!("AGENTHUB_REMOTE_RUNNER={}", runner.id)])
        .args(["-e", &format!("AGENTHUB_SANDBOX_LEVEL={sandbox_level}")]);
    apply_resource_args(&mut process);
    process
        .arg(image)
        .arg("sh")
        .arg("-lc")
        .arg(shell_command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok(process)
}

pub(super) fn is_endpoint(endpoint: &str) -> bool {
    endpoint.starts_with("docker://")
}

fn apply_resource_args(process: &mut Command) {
    if let Ok(cpu) = std::env::var("AGENTHUB_CPU_CORES") {
        process.args(["--cpus", &cpu]);
    }
    if let Ok(memory) = std::env::var("AGENTHUB_MEMORY_MB") {
        process.args(["--memory", &format!("{memory}m")]);
    }
    if let Ok(network) = std::env::var("AGENTHUB_NETWORK_MODE") {
        if matches!(network.as_str(), "none" | "host" | "bridge") {
            process.args(["--network", &network]);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use anyhow::Result;

    use super::super::metadata::metadata_for;
    use super::*;

    #[test]
    fn docker_endpoint_builds_container_command() -> Result<()> {
        std::env::set_var("AGENTHUB_CPU_CORES", "1.5");
        std::env::set_var("AGENTHUB_MEMORY_MB", "512");
        std::env::set_var("AGENTHUB_NETWORK_MODE", "none");
        let runner = RemoteRunner {
            id: "docker-test".to_string(),
            endpoint: "docker://rust:latest".to_string(),
        };

        let command = command("cargo test", Path::new("/tmp/project"), 2, &runner)?;
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        std::env::remove_var("AGENTHUB_CPU_CORES");
        std::env::remove_var("AGENTHUB_MEMORY_MB");
        std::env::remove_var("AGENTHUB_NETWORK_MODE");
        assert_eq!(command.get_program().to_string_lossy(), "docker");
        assert!(args.contains(&"rust:latest".to_string()));
        assert!(args.contains(&"--cpus".to_string()));
        assert!(args.contains(&"1.5".to_string()));
        assert!(args.contains(&"--memory".to_string()));
        assert!(args.contains(&"512m".to_string()));
        assert!(args.contains(&"--network".to_string()));
        assert!(args.contains(&"none".to_string()));
        Ok(())
    }

    #[test]
    fn docker_metadata_reports_container_capabilities() {
        let runner = RemoteRunner {
            id: "docker-test".to_string(),
            endpoint: "docker://rust:latest".to_string(),
        };

        let metadata = metadata_for(2, Some(&runner), Duration::from_secs(30));

        assert_eq!(metadata.kind, "docker");
        assert_eq!(metadata.trust_level, "team-runner");
        assert!(metadata
            .capabilities
            .contains(&"container_backend".to_string()));
        assert!(metadata.capabilities.contains(&"cpu_limit".to_string()));
        assert!(metadata.capabilities.contains(&"memory_limit".to_string()));
    }
}
