use anyhow::{anyhow, Result};

use crate::aal::diagnostics::AalDiagnostic;
use crate::aal::draft::{AalImport, Draft};

pub(crate) fn handle(draft: &mut Draft, line: usize, tokens: &[String]) -> Result<bool> {
    match tokens.first().map(String::as_str) {
        Some("aal") if tokens.len() == 2 => {
            draft.version = Some(tokens[1].clone());
            Ok(true)
        }
        Some("import") if tokens.len() == 3 => {
            import(draft, line, &tokens[1], &tokens[2])?;
            Ok(true)
        }
        Some("aal") | Some("import") => Err(anyhow!(
            "{}",
            AalDiagnostic::error(line, "invalid AAL preamble statement").render()
        )),
        _ => Ok(false),
    }
}

fn import(draft: &mut Draft, line: usize, kind: &str, raw: &str) -> Result<()> {
    let kind = normalize_kind(line, kind)?;
    let (id, version) = raw
        .split_once('@')
        .map(|(id, version)| (id.to_string(), Some(version.to_string())))
        .unwrap_or_else(|| (raw.to_string(), None));
    draft.imports.push(AalImport {
        kind: kind.to_string(),
        id,
        version,
        line,
    });
    Ok(())
}

fn normalize_kind(line: usize, kind: &str) -> Result<&'static str> {
    match kind {
        "skill" => Ok("skill"),
        "rule" | "rules" => Ok("rules"),
        _ => Err(anyhow!(
            "{}",
            AalDiagnostic::error(line, format!("unsupported import kind `{kind}`")).render()
        )),
    }
}
