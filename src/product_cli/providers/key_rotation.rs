use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

#[derive(Debug, Default)]
pub struct KeyRotationOptions {
    pub from_file: Option<PathBuf>,
    pub from_env: Option<String>,
    pub stdin_value: Option<String>,
    pub target: Option<PathBuf>,
    pub dry_run: bool,
    pub test_after_install: bool,
}

#[derive(Debug)]
pub struct KeyRotationResult {
    pub output: String,
    pub provider_test_failed: bool,
}

pub fn rotate_provider_key(
    project_root: &Path,
    provider: &str,
    options: KeyRotationOptions,
) -> Result<KeyRotationResult> {
    if provider != "kimi" {
        return Err(anyhow!(
            "provider key rotation is only supported for `kimi` right now"
        ));
    }
    let source_count = usize::from(options.from_file.is_some())
        + usize::from(options.from_env.is_some())
        + usize::from(options.stdin_value.is_some());
    if source_count > 1 {
        return Err(anyhow!("choose exactly one key source"));
    }

    let target = kimi_key_rotation_target(project_root, options.target.as_deref())?;
    let (raw_key, source) = rotation_source(&options)?;
    let new_key = raw_key.trim().to_string();
    let trimmed_for_write = raw_key != new_key;
    if new_key.is_empty() {
        return Err(anyhow!("replacement key is empty after trimming"));
    }
    if new_key.chars().any(char::is_whitespace) {
        return Err(anyhow!(
            "replacement key contains embedded whitespace after trimming"
        ));
    }

    let old_key = fs::read_to_string(&target)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let old_fp = old_key
        .as_deref()
        .map(|key| super::sha256_prefix(key.as_bytes()))
        .unwrap_or_else(|| "none".to_string());
    let new_fp = super::sha256_prefix(new_key.as_bytes());

    let mut out = String::from("AgentHub Kimi key rotation\n");
    out.push_str("provider\tkimi\n");
    out.push_str(&format!("target\t{}\n", target.display()));
    out.push_str(&format!("source\t{source}\n"));
    out.push_str(&format!("old_key_sha256_12\t{old_fp}\n"));
    out.push_str(&format!("new_key_sha256_12\t{new_fp}\n"));
    out.push_str(&format!("new_key_chars\t{}\n", new_key.chars().count()));
    out.push_str(&format!("trimmed_for_write\t{trimmed_for_write}\n"));
    append_active_key_warning(&mut out, &target, &new_key);

    if options.dry_run {
        out.push_str("status\tdry_run\n");
        out.push_str("next\t1\tagenthub providers rotate-key kimi --from-file <new-key-file>\n");
        return Ok(KeyRotationResult {
            output: out,
            provider_test_failed: false,
        });
    }

    install_key_atomically(&target, &new_key)?;
    out.push_str("status\tinstalled\n");
    out.push_str("next\t1\tagenthub providers rc-unblock kimi\n");
    out.push_str("next\t2\tscripts/kimi-rc-unblock.sh\n");
    out.push_str("next\t3\tagenthub providers test kimi\n");
    out.push_str("next\t4\tscripts/kimi-auth-check.sh\n");
    out.push_str("next\t5\tAGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh\n");
    out.push_str("next\t6\tscripts/rc-evidence-collect.sh\n");
    out.push_str("next\t7\tscripts/rc-dogfood-gate.sh --check\n");

    let mut provider_test_failed = false;
    if options.test_after_install {
        let report = super::test_provider(project_root, "kimi")?;
        provider_test_failed = super::test_report_failed(&report);
        out.push_str("provider_test\tbegin\n");
        for line in report.lines() {
            out.push_str(&format!("provider_test\t{line}\n"));
        }
    }

    Ok(KeyRotationResult {
        output: out,
        provider_test_failed,
    })
}

fn rotation_source(options: &KeyRotationOptions) -> Result<(String, String)> {
    if let Some(path) = &options.from_file {
        let value = fs::read_to_string(path).map_err(|error| {
            anyhow!(
                "failed to read source key file `{}`: {error}",
                path.display()
            )
        })?;
        return Ok((value, format!("file:{}", path.display())));
    }
    if let Some(env_name) = &options.from_env {
        validate_kimi_key_env(env_name)?;
        let value = std::env::var(env_name)
            .map_err(|_| anyhow!("environment variable `{env_name}` is not set"))?;
        return Ok((value, format!("env:{env_name}")));
    }
    if let Some(value) = &options.stdin_value {
        return Ok((value.clone(), "stdin".to_string()));
    }
    for env_name in ["KIMI_API_KEY", "MOONSHOT_API_KEY"] {
        if let Ok(value) = std::env::var(env_name) {
            if !value.trim().is_empty() {
                return Ok((value, format!("env:{env_name}")));
            }
        }
    }
    Err(anyhow!(
        "missing key source; pass --from-file, --from-env, --stdin, KIMI_API_KEY, or MOONSHOT_API_KEY"
    ))
}

fn validate_kimi_key_env(env_name: &str) -> Result<()> {
    match env_name {
        "KIMI_API_KEY" | "MOONSHOT_API_KEY" => Ok(()),
        _ => Err(anyhow!(
            "unsupported key env `{env_name}`; use KIMI_API_KEY or MOONSHOT_API_KEY"
        )),
    }
}

fn kimi_key_rotation_target(project_root: &Path, explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    if let Some(path) = std::env::var_os("AGENTHUB_KIMI_KEY_FILE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(path);
    }
    if let Some(path) = super::status_for(project_root, "kimi")?.api_key_file {
        return Ok(path);
    }
    Ok(project_root.join(".kimi"))
}

fn append_active_key_warning(out: &mut String, target: &Path, new_key: &str) {
    for env_name in ["KIMI_API_KEY", "MOONSHOT_API_KEY"] {
        let Ok(value) = std::env::var(env_name) else {
            continue;
        };
        let active_key = value.trim();
        if active_key.is_empty() {
            continue;
        }
        out.push_str(&format!("active_key_source_after\tenv:{env_name}\n"));
        if active_key != new_key {
            out.push_str(&format!(
                "warning\tenv_key_overrides_target_file\t{env_name}\t{}\n",
                target.display()
            ));
        }
        return;
    }
    out.push_str(&format!(
        "active_key_source_after\tfile:{}\n",
        target.display()
    ));
}

fn install_key_atomically(target: &Path, key: &str) -> Result<()> {
    let target_dir = target
        .parent()
        .ok_or_else(|| anyhow!("target key path has no parent: {}", target.display()))?;
    fs::create_dir_all(target_dir)?;
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(".kimi");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let tmp = target_dir.join(format!(".{file_name}.tmp.{}.{}", std::process::id(), nonce));

    let write_result = (|| -> Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options.open(&tmp)?;
        writeln!(file, "{key}")?;
        file.sync_all()?;
        fs::rename(&tmp, target)?;
        #[cfg(unix)]
        fs::set_permissions(target, fs::Permissions::from_mode(0o600))?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    write_result
}
