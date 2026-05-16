use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use agenthub::{aal, compiler};

use crate::cli::AalCommands;

#[cfg(test)]
mod tests;

pub fn handle_aal(command: AalCommands) -> Result<()> {
    match command {
        AalCommands::Parse { input, output } => parse(&input, output.as_deref()),
        AalCommands::Format {
            input,
            output,
            check,
        } => format(&input, output.as_deref(), check),
        AalCommands::Check {
            input,
            expected_dir,
            write_expected,
        } => check(&input, expected_dir.as_deref(), write_expected),
    }
}

fn parse(input: &Path, output: Option<&Path>) -> Result<()> {
    let parsed = aal::parse_aal_file(input)?;
    print_diagnostics(input, &parsed);
    if parsed.has_errors() {
        bail!("AAL semantic validation failed");
    }
    let yaml = serde_yaml::to_string(&parsed.spec)?;
    if let Some(output) = output {
        write_output(output, &yaml)?;
        println!("{}", output.display());
    } else {
        print!("{yaml}");
    }
    Ok(())
}

fn format(input: &Path, output: Option<&Path>, check: bool) -> Result<()> {
    let source = fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
    let formatted = aal::format_aal(&source)?;
    if check {
        if normalize_newlines(&source) != normalize_newlines(&formatted) {
            bail!("AAL format mismatch: {}", input.display());
        }
        println!("ok\t{}", input.display());
        return Ok(());
    }
    if let Some(output) = output {
        write_output(output, &formatted)?;
        println!("{}", output.display());
    } else {
        print!("{formatted}");
    }
    Ok(())
}

fn check(input: &Path, expected_dir: Option<&Path>, write_expected: bool) -> Result<()> {
    let result = check_artifacts(input, expected_dir, write_expected)?;
    println!("ok\t{}", input.display());
    println!("task\t{}", result.task_id);
    println!("dag_nodes\t{}", result.dag_nodes);
    println!("dag_edges\t{}", result.dag_edges);
    if let Some(dir) = result.expected_dir {
        println!("expected\tmatched\t{}", dir.display());
    }
    Ok(())
}

struct CheckResult {
    task_id: String,
    dag_nodes: usize,
    dag_edges: usize,
    expected_dir: Option<PathBuf>,
}

fn check_artifacts(
    input: &Path,
    expected_dir: Option<&Path>,
    write_expected: bool,
) -> Result<CheckResult> {
    let parsed = aal::parse_aal_file(input)?;
    print_diagnostics(input, &parsed);
    if parsed.has_errors() {
        bail!("AAL semantic validation failed");
    }
    let dag = compiler::compile(&parsed.spec)?;
    let yaml = serde_yaml::to_string(&parsed.spec)?;
    let ir = parsed.spec.to_agent_ir();
    let dag_json = serde_json::to_string_pretty(&dag)?;
    let expected = resolve_expected_dir(input, expected_dir, write_expected);
    if write_expected {
        let dir = expected
            .as_deref()
            .context("expected directory could not be resolved")?;
        write_expected_artifacts(input, dir, &yaml, &ir, &dag_json)?;
    } else if let Some(dir) = expected.as_deref() {
        compare_expected(input, dir, &yaml, &ir, &dag_json)?;
    }
    Ok(CheckResult {
        task_id: parsed.spec.task.id,
        dag_nodes: dag.nodes.len(),
        dag_edges: dag.edges.len(),
        expected_dir: expected,
    })
}

fn resolve_expected_dir(
    input: &Path,
    explicit: Option<&Path>,
    write_expected: bool,
) -> Option<PathBuf> {
    explicit.map(Path::to_path_buf).or_else(|| {
        let dir = input.parent()?.join("expected");
        (write_expected || dir.exists()).then_some(dir)
    })
}

fn compare_expected(input: &Path, dir: &Path, yaml: &str, ir: &str, dag_json: &str) -> Result<()> {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .context("AAL input must have a UTF-8 file stem")?;
    compare_file(&dir.join(format!("{stem}.yaml")), yaml)?;
    compare_file(&dir.join(format!("{stem}.ir")), ir)?;
    compare_file(&dir.join(format!("{stem}.dag.json")), dag_json)?;
    Ok(())
}

fn write_expected_artifacts(
    input: &Path,
    dir: &Path,
    yaml: &str,
    ir: &str,
    dag_json: &str,
) -> Result<()> {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .context("AAL input must have a UTF-8 file stem")?;
    fs::create_dir_all(dir)?;
    write_output(&dir.join(format!("{stem}.yaml")), yaml)?;
    write_output(&dir.join(format!("{stem}.ir")), ir)?;
    write_output(&dir.join(format!("{stem}.dag.json")), dag_json)?;
    Ok(())
}

fn compare_file(path: &Path, actual: &str) -> Result<()> {
    let expected = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if normalize_newlines(&expected) != normalize_newlines(actual) {
        bail!("AAL golden mismatch: {}", path.display());
    }
    Ok(())
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n")
}

fn write_output(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn print_diagnostics(input: &Path, parsed: &aal::AalParseOutput) {
    let source = fs::read_to_string(input).unwrap_or_default();
    let lines = source.lines().collect::<Vec<_>>();
    for diagnostic in &parsed.diagnostics {
        eprintln!("{}", diagnostic.render());
        if diagnostic.line > 0 {
            if let Some(line) = lines.get(diagnostic.line - 1) {
                eprintln!("  {}", line.trim_end());
            }
        }
        if let Some(help) = &diagnostic.help {
            eprintln!("  help: {help}");
        }
    }
}
