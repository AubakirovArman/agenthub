use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{agent_dir, chat_index, product_cli::providers};

use super::line_editor::{slash_display, slash_matches};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CompletionCandidate {
    pub display: String,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CompletionSet {
    pub start: usize,
    pub candidates: Vec<CompletionCandidate>,
}

pub(super) fn complete(root: &Path, line: &str, pos: usize) -> Result<CompletionSet> {
    let safe_pos = pos.min(line.len());
    let prefix = &line[..safe_pos];
    if prefix.starts_with('/') && !prefix.contains(' ') {
        return Ok(CompletionSet {
            start: 0,
            candidates: slash_matches(prefix)
                .into_iter()
                .map(|command| CompletionCandidate {
                    display: slash_display(command),
                    replacement: command.to_string(),
                })
                .collect(),
        });
    }
    if let Some((start, token)) = mention_token(prefix) {
        return Ok(CompletionSet {
            start,
            candidates: mention_candidates(root, token)?,
        });
    }
    if let Some((start, token)) = provider_token(prefix) {
        return Ok(CompletionSet {
            start,
            candidates: provider_candidates(root, token)?,
        });
    }
    Ok(CompletionSet {
        start: safe_pos,
        candidates: Vec::new(),
    })
}

pub(super) fn hint(root: &Path, line: &str, pos: usize) -> Option<String> {
    let set = complete(root, line, pos).ok()?;
    if set.candidates.len() != 1 {
        return None;
    }
    let candidate = &set.candidates[0].replacement;
    candidate
        .strip_prefix(&line[set.start..pos.min(line.len())])
        .map(str::to_string)
}

fn mention_token(prefix: &str) -> Option<(usize, &str)> {
    let start = prefix.rfind('@')?;
    if prefix[start + 1..].chars().any(char::is_whitespace) {
        return None;
    }
    Some((start, &prefix[start + 1..]))
}

fn provider_token(prefix: &str) -> Option<(usize, &str)> {
    for command in ["/providers ", "/provider ", "/run --provider "] {
        if let Some(rest) = prefix.strip_prefix(command) {
            if !rest.chars().any(char::is_whitespace) {
                return Some((command.len(), rest));
            }
        }
    }
    None
}

fn mention_candidates(root: &Path, token: &str) -> Result<Vec<CompletionCandidate>> {
    let mut candidates = Vec::new();
    if token.starts_with("tx:") {
        let query = token.trim_start_matches("tx:");
        candidates.extend(
            agent_dir::list_transactions(root)?
                .into_iter()
                .rev()
                .filter(|row| row.id.contains(query) || query.is_empty())
                .take(20)
                .map(|row| CompletionCandidate {
                    display: format!("tx:{}\t{}", row.id, row.status),
                    replacement: format!("@tx:{}", row.id),
                }),
        );
        candidates.insert(
            0,
            CompletionCandidate {
                display: "tx:latest\tlatest transaction".to_string(),
                replacement: "@tx:latest".to_string(),
            },
        );
        return Ok(candidates);
    }
    if token.starts_with("chat:") {
        let query = token.trim_start_matches("chat:");
        candidates.extend(
            chat_index::list(root, 20)?
                .into_iter()
                .filter(|row| row.id.contains(query) || row.title.contains(query))
                .map(|row| CompletionCandidate {
                    display: format!("chat:{}\t{}", row.id, row.title),
                    replacement: format!("@chat:{}", row.id),
                }),
        );
        return Ok(candidates);
    }
    if token.starts_with("memory:") {
        for item in ["memory:summary", "memory:decisions", "memory:failures"] {
            if item.starts_with(token) {
                candidates.push(CompletionCandidate {
                    display: item.to_string(),
                    replacement: format!("@{item}"),
                });
            }
        }
        return Ok(candidates);
    }
    candidates.extend(path_candidates(root, token)?);
    for special in ["tx:latest", "chat:latest", "memory:summary"] {
        if special.starts_with(token) {
            candidates.push(CompletionCandidate {
                display: special.to_string(),
                replacement: format!("@{special}"),
            });
        }
    }
    Ok(candidates)
}

fn provider_candidates(root: &Path, token: &str) -> Result<Vec<CompletionCandidate>> {
    Ok(providers::statuses(root)?
        .into_iter()
        .filter(|status| status.info.id.starts_with(token))
        .map(|status| CompletionCandidate {
            display: format!(
                "{}\t{}",
                status.info.id,
                if status.available { "ready" } else { "missing" }
            ),
            replacement: status.info.id,
        })
        .collect())
}

fn path_candidates(root: &Path, token: &str) -> Result<Vec<CompletionCandidate>> {
    let token_path = Path::new(token);
    let (dir, leaf) = match token.rsplit_once('/') {
        Some((parent, leaf)) => (root.join(parent), leaf),
        None => (root.to_path_buf(), token),
    };
    let mut candidates = Vec::new();
    if !dir.is_dir() {
        return Ok(candidates);
    }
    for entry in fs::read_dir(&dir)?.take(80) {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(leaf) || should_skip(&name) {
            continue;
        }
        let relative = if token_path.parent().is_some() {
            PathBuf::from(token)
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(&name)
        } else {
            PathBuf::from(&name)
        };
        let mut value = relative.display().to_string();
        if entry.file_type()?.is_dir() {
            value.push('/');
        }
        candidates.push(CompletionCandidate {
            display: value.clone(),
            replacement: format!("@{}", quote_if_needed(&value)),
        });
    }
    candidates.sort_by(|a, b| a.display.cmp(&b.display));
    candidates.truncate(30);
    Ok(candidates)
}

fn quote_if_needed(value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | ".agent" | ".venv" | "__pycache__"
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn completes_paths_and_quotes_spaces() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::create_dir_all(dir.path().join("docs"))?;
        fs::write(dir.path().join("docs/path with spaces.md"), "demo")?;

        let set = complete(dir.path(), "open @docs/path", "open @docs/path".len())?;

        assert_eq!(set.start, "open ".len());
        assert!(set
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "@\"docs/path with spaces.md\""));
        Ok(())
    }

    #[test]
    fn completes_slash_commands() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let set = complete(dir.path(), "/pro", 4)?;

        assert_eq!(set.start, 0);
        assert!(set
            .candidates
            .iter()
            .any(|candidate| candidate.replacement == "/providers"));
        Ok(())
    }
}
