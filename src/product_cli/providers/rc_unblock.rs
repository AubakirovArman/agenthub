use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Result};

#[derive(Debug, Default)]
pub struct RcUnblockOptions {
    pub skip_provider_dogfood: bool,
    pub no_check: bool,
    pub rotate_key: Option<super::KeyRotationOptions>,
}

#[derive(Debug)]
pub struct RcUnblockResult {
    pub output: String,
    pub failed: bool,
}

pub fn rc_unblock_provider(
    project_root: &Path,
    provider: &str,
    options: RcUnblockOptions,
) -> Result<RcUnblockResult> {
    if provider != "kimi" {
        return Err(anyhow!(
            "provider RC unblock is only supported for `kimi` right now"
        ));
    }

    let mut out = String::from("AgentHub Kimi RC unblock\n");
    out.push_str("provider\tkimi\n");

    let mut endpoint_override = None;
    if let Some(rotation_options) = options.rotate_key {
        if let Some(preflight_options) = preflight_options_from_rotation(&rotation_options) {
            let preflight = run_key_preflight(project_root, &mut out, preflight_options)?;
            if preflight.provider_test_failed {
                return blocked_result(
                    project_root,
                    out,
                    "key_preflight_failed",
                    endpoint_override.as_deref(),
                );
            }
            if let Some(passed_endpoint) = preflight.passed_endpoint {
                if preflight.configured_endpoint.as_deref() != Some(passed_endpoint.as_str()) {
                    out.push_str(&format!(
                        "endpoint_override\tKIMI_API_BASE_URL\t{passed_endpoint}\n"
                    ));
                    endpoint_override = Some(passed_endpoint);
                }
            }
        }
        if !run_key_rotation(project_root, &mut out, rotation_options)? {
            return blocked_result(
                project_root,
                out,
                "key_rotation_provider_test_failed",
                endpoint_override.as_deref(),
            );
        }
    }

    if !run_provider_test(project_root, &mut out, endpoint_override.as_deref())? {
        run_auth_check_after_provider_failure(
            project_root,
            &mut out,
            endpoint_override.as_deref(),
        )?;
        return blocked_result(
            project_root,
            out,
            "provider_test_failed",
            endpoint_override.as_deref(),
        );
    }

    if !run_script(
        project_root,
        &mut out,
        "kimi_auth_check",
        &script(project_root, "kimi-auth-check.sh"),
        &[],
        &[],
        endpoint_override.as_deref(),
    )? {
        return blocked_result(
            project_root,
            out,
            "kimi_auth_check_failed",
            endpoint_override.as_deref(),
        );
    }

    if options.skip_provider_dogfood {
        out.push_str("step\tprovider_dogfood\tskipped\n");
        out.push_str("warning\tprovider_dogfood_required_for_rc_gate\n");
    } else if !run_script(
        project_root,
        &mut out,
        "provider_dogfood",
        &script(project_root, "provider-dogfood.sh"),
        &[],
        &[
            ("AGENTHUB_PROVIDER_DOGFOOD_PROVIDER", "kimi"),
            ("AGENTHUB_PROVIDER_DOGFOOD_LIVE", "1"),
        ],
        endpoint_override.as_deref(),
    )? {
        out.push_str("next\t1\tAGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh\n");
        return blocked_result(
            project_root,
            out,
            "provider_dogfood_failed",
            endpoint_override.as_deref(),
        );
    }

    if !run_script(
        project_root,
        &mut out,
        "rc_evidence_collect",
        &script(project_root, "rc-evidence-collect.sh"),
        &[],
        &[],
        endpoint_override.as_deref(),
    )? {
        return blocked_result(
            project_root,
            out,
            "rc_evidence_collect_failed",
            endpoint_override.as_deref(),
        );
    }

    let gate_args = if options.no_check {
        Vec::new()
    } else {
        vec!["--check"]
    };
    let gate_label = if options.no_check {
        "rc_dogfood_gate_summary"
    } else {
        "rc_dogfood_gate"
    };
    let gate_passed = run_script(
        project_root,
        &mut out,
        gate_label,
        &script(project_root, "rc-dogfood-gate.sh"),
        &gate_args,
        &[],
        endpoint_override.as_deref(),
    )?;
    if !gate_passed {
        super::operator_receipt::append_kimi_rc_blocked_operator_receipt(
            project_root,
            &mut out,
            endpoint_override.as_deref(),
            "rc_dogfood_gate_failed",
        )?;
        append_blocked(&mut out, "rc_dogfood_gate_failed");
        return Ok(RcUnblockResult {
            output: out,
            failed: true,
        });
    }
    super::operator_receipt::append_kimi_rc_operator_receipt(
        project_root,
        &mut out,
        endpoint_override.as_deref(),
    )?;

    if options.no_check {
        out.push_str("status\tunchecked\n");
        out.push_str("next\t1\tscripts/rc-dogfood-gate.sh --check\n");
    } else {
        out.push_str("status\tready\n");
    }
    Ok(RcUnblockResult {
        output: out,
        failed: false,
    })
}

fn preflight_options_from_rotation(
    options: &super::KeyRotationOptions,
) -> Option<super::KeyPreflightOptions> {
    if options.from_file.is_none() && options.from_env.is_none() && options.stdin_value.is_none() {
        return None;
    }
    Some(super::KeyPreflightOptions {
        from_file: options.from_file.clone(),
        from_env: options.from_env.clone(),
        stdin_value: options.stdin_value.clone(),
    })
}

fn run_key_preflight(
    project_root: &Path,
    out: &mut String,
    options: super::KeyPreflightOptions,
) -> Result<super::KeyPreflightResult> {
    out.push_str("step\tkey_preflight\tbegin\n");
    let result = super::preflight_provider_key(project_root, "kimi", options)?;
    for line in result.output.lines() {
        out.push_str(&format!("key_preflight\t{line}\n"));
    }
    if result.provider_test_failed {
        out.push_str("step\tkey_preflight\tfailed\n");
    } else {
        out.push_str("step\tkey_preflight\tpassed\n");
    }
    Ok(result)
}

fn run_key_rotation(
    project_root: &Path,
    out: &mut String,
    options: super::KeyRotationOptions,
) -> Result<bool> {
    out.push_str("step\tkey_rotation\tbegin\n");
    let result = super::rotate_provider_key(project_root, "kimi", options)?;
    for line in result.output.lines() {
        out.push_str(&format!("key_rotation\t{line}\n"));
    }
    if result.provider_test_failed {
        out.push_str("step\tkey_rotation\tfailed\tprovider_test_failed\n");
        Ok(false)
    } else {
        out.push_str("step\tkey_rotation\tpassed\n");
        Ok(true)
    }
}

fn run_provider_test(
    project_root: &Path,
    out: &mut String,
    endpoint_override: Option<&str>,
) -> Result<bool> {
    out.push_str("step\tprovider_test\tbegin\n");
    let report = if let Some(endpoint) = endpoint_override {
        let mut status = super::status_for(project_root, "kimi")?;
        status.endpoint = Some(endpoint.to_string());
        if super::api_key_for_status(&status).is_none() {
            format!(
                "missing\t{}\t{}\n",
                status.info.id,
                super::status_detail(&status)
            )
        } else {
            super::http::test_provider(status)?
        }
    } else {
        super::test_provider(project_root, "kimi")?
    };
    for line in report.lines() {
        out.push_str(&format!("provider_test\t{line}\n"));
    }
    if super::test_report_failed(&report) {
        out.push_str("step\tprovider_test\tfailed\n");
        Ok(false)
    } else {
        out.push_str("step\tprovider_test\tpassed\n");
        Ok(true)
    }
}

fn run_auth_check_after_provider_failure(
    project_root: &Path,
    out: &mut String,
    endpoint_override: Option<&str>,
) -> Result<()> {
    let path = script(project_root, "kimi-auth-check.sh");
    if path.exists() {
        let _ = run_script(
            project_root,
            out,
            "kimi_auth_check",
            &path,
            &[],
            &[],
            endpoint_override,
        )?;
    } else {
        out.push_str("step\tkimi_auth_check\tskipped\tmissing_script\n");
    }
    Ok(())
}

fn run_script(
    project_root: &Path,
    out: &mut String,
    label: &str,
    path: &Path,
    args: &[&str],
    envs: &[(&str, &str)],
    endpoint_override: Option<&str>,
) -> Result<bool> {
    out.push_str(&format!("step\t{label}\tbegin\n"));
    out.push_str(&format!("script\t{label}\t{}\n", path.display()));
    if !path.exists() {
        out.push_str(&format!("step\t{label}\tfailed\tmissing_script\n"));
        out.push_str(&format!(
            "next\t1\tcd {} && scripts/kimi-rc-unblock.sh\n",
            project_root.display()
        ));
        return Ok(false);
    }

    let mut command = Command::new(path);
    command.current_dir(project_root).args(args);
    if let Some(endpoint) = endpoint_override {
        command.env("KIMI_API_BASE_URL", endpoint);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output()?;
    append_command_output(out, label, "stdout", &output.stdout);
    append_command_output(out, label, "stderr", &output.stderr);
    if output.status.success() {
        out.push_str(&format!("step\t{label}\tpassed\n"));
        Ok(true)
    } else {
        let code = output
            .status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_string());
        out.push_str(&format!("step\t{label}\tfailed\t{code}\n"));
        Ok(false)
    }
}

fn append_command_output(out: &mut String, label: &str, stream: &str, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    for line in text.lines() {
        out.push_str(&format!("{label}\t{stream}\t{line}\n"));
    }
}

fn append_blocked(out: &mut String, reason: &str) {
    out.push_str("status\tblocked\n");
    out.push_str(&format!("reason\t{reason}\n"));
    out.push_str("next\t1\tagenthub providers inspect-key kimi\n");
    out.push_str("next\t2\tagenthub providers inspect-key kimi --from-file <new-key-file>\n");
    out.push_str("next\t3\tagenthub providers rehearse-unblock kimi --from-file <new-key-file>\n");
    out.push_str("next\t4\tagenthub providers preflight-key kimi --from-file <new-key-file>\n");
    out.push_str("next\t5\tagenthub providers rc-unblock kimi --from-file <new-key-file>\n");
    out.push_str("next\t6\tagenthub providers rotate-key kimi --from-file <new-key-file>\n");
    out.push_str("next\t7\tagenthub providers unblock kimi\n");
}

fn blocked_result(
    project_root: &Path,
    mut out: String,
    reason: &str,
    endpoint_override: Option<&str>,
) -> Result<RcUnblockResult> {
    super::operator_receipt::append_kimi_rc_blocked_operator_receipt(
        project_root,
        &mut out,
        endpoint_override,
        reason,
    )?;
    append_blocked(&mut out, reason);
    Ok(RcUnblockResult {
        output: out,
        failed: true,
    })
}

fn script(project_root: &Path, name: &str) -> PathBuf {
    project_root.join("scripts").join(name)
}
