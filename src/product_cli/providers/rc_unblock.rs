use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Result};

#[derive(Debug, Default)]
pub struct RcUnblockOptions {
    pub skip_provider_dogfood: bool,
    pub no_check: bool,
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

    if !run_provider_test(project_root, &mut out)? {
        append_blocked(&mut out, "provider_test_failed");
        return Ok(RcUnblockResult {
            output: out,
            failed: true,
        });
    }

    if !run_script(
        project_root,
        &mut out,
        "kimi_auth_check",
        &script(project_root, "kimi-auth-check.sh"),
        &[],
        &[],
    )? {
        append_blocked(&mut out, "kimi_auth_check_failed");
        return Ok(RcUnblockResult {
            output: out,
            failed: true,
        });
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
    )? {
        append_blocked(&mut out, "provider_dogfood_failed");
        out.push_str("next\t1\tAGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh\n");
        return Ok(RcUnblockResult {
            output: out,
            failed: true,
        });
    }

    if !run_script(
        project_root,
        &mut out,
        "rc_evidence_collect",
        &script(project_root, "rc-evidence-collect.sh"),
        &[],
        &[],
    )? {
        append_blocked(&mut out, "rc_evidence_collect_failed");
        return Ok(RcUnblockResult {
            output: out,
            failed: true,
        });
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
    if !run_script(
        project_root,
        &mut out,
        gate_label,
        &script(project_root, "rc-dogfood-gate.sh"),
        &gate_args,
        &[],
    )? {
        append_blocked(&mut out, "rc_dogfood_gate_failed");
        return Ok(RcUnblockResult {
            output: out,
            failed: true,
        });
    }

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

fn run_provider_test(project_root: &Path, out: &mut String) -> Result<bool> {
    out.push_str("step\tprovider_test\tbegin\n");
    let report = super::test_provider(project_root, "kimi")?;
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

fn run_script(
    project_root: &Path,
    out: &mut String,
    label: &str,
    path: &Path,
    args: &[&str],
    envs: &[(&str, &str)],
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
    out.push_str("next\t1\tagenthub providers rotate-key kimi --from-file <new-key-file>\n");
    out.push_str("next\t2\tagenthub providers unblock kimi\n");
}

fn script(project_root: &Path, name: &str) -> PathBuf {
    project_root.join("scripts").join(name)
}
