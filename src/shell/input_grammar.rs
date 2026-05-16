#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum InputKind {
    Task,
    Slash,
    MentionOnly,
    Shell,
    MemoryNote,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MentionKind {
    File,
    Directory,
    Transaction,
    Chat,
    Memory,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MentionToken {
    pub raw: String,
    pub start: usize,
    pub end: usize,
    pub kind: MentionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedInput {
    pub kind: InputKind,
    pub clean_text: String,
    pub mentions: Vec<MentionToken>,
}

pub(super) fn parse(line: &str) -> ParsedInput {
    let trimmed = line.trim();
    let kind = if trimmed.is_empty() {
        InputKind::Empty
    } else if trimmed.starts_with('/') {
        InputKind::Slash
    } else if trimmed.starts_with('!') {
        InputKind::Shell
    } else if trimmed.starts_with('#') {
        InputKind::MemoryNote
    } else if trimmed.starts_with('@') && next_non_mention_text(line).is_empty() {
        InputKind::MentionOnly
    } else {
        InputKind::Task
    };
    let (clean_text, mentions) = strip_mentions(line);
    ParsedInput {
        kind,
        clean_text,
        mentions,
    }
}

pub(super) fn strip_mentions(line: &str) -> (String, Vec<MentionToken>) {
    let mut clean = String::new();
    let mut mentions = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch != '@' {
            clean.push(ch);
            continue;
        }
        let start = index;
        let Some((next_index, next)) = chars.peek().copied() else {
            clean.push('@');
            continue;
        };
        if next.is_whitespace() {
            clean.push('@');
            continue;
        }
        let (raw, end) = if matches!(next, '"' | '\'') {
            chars.next();
            read_quoted(next, &mut chars).unwrap_or_else(|| {
                clean.push('@');
                (String::new(), next_index)
            })
        } else {
            read_unquoted(&mut chars, next_index, next)
        };
        if raw.is_empty() {
            continue;
        }
        mentions.push(MentionToken {
            kind: mention_kind(&raw),
            raw,
            start,
            end,
        });
        if !clean.chars().last().is_some_and(|ch| ch.is_whitespace()) {
            clean.push(' ');
        }
    }
    (squash_ws(&clean), mentions)
}

fn read_quoted<I>(quote: char, chars: &mut std::iter::Peekable<I>) -> Option<(String, usize)>
where
    I: Iterator<Item = (usize, char)>,
{
    let mut raw = String::new();
    let mut escaped = false;
    for (index, ch) in chars.by_ref() {
        if escaped {
            raw.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some((raw, index + ch.len_utf8()));
        }
        raw.push(ch);
    }
    None
}

fn read_unquoted<I>(
    chars: &mut std::iter::Peekable<I>,
    first_index: usize,
    first: char,
) -> (String, usize)
where
    I: Iterator<Item = (usize, char)>,
{
    let mut raw = String::new();
    let mut end = first_index + first.len_utf8();
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_whitespace() {
            break;
        }
        chars.next();
        end = index + ch.len_utf8();
        if ch == '\\' {
            if let Some((escaped_index, escaped)) = chars.next() {
                raw.push(escaped);
                end = escaped_index + escaped.len_utf8();
            }
        } else {
            raw.push(ch);
        }
    }
    (raw, end)
}

fn mention_kind(raw: &str) -> MentionKind {
    if raw.starts_with("tx:") || raw.starts_with("transaction:") {
        MentionKind::Transaction
    } else if raw.starts_with("chat:") {
        MentionKind::Chat
    } else if raw.starts_with("memory:") {
        MentionKind::Memory
    } else if raw.ends_with('/') {
        MentionKind::Directory
    } else if raw.contains('.') || raw.contains('/') {
        MentionKind::File
    } else {
        MentionKind::Unknown
    }
}

fn next_non_mention_text(line: &str) -> String {
    strip_mentions(line).0
}

fn squash_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_prefix_input_kinds() {
        assert_eq!(parse("/help").kind, InputKind::Slash);
        assert_eq!(parse("!cargo test").kind, InputKind::Shell);
        assert_eq!(parse("# remember this").kind, InputKind::MemoryNote);
        assert_eq!(parse("@src/lib.rs").kind, InputKind::MentionOnly);
        assert_eq!(parse("change @src/lib.rs").kind, InputKind::Task);
    }

    #[test]
    fn extracts_quoted_and_escaped_mentions() {
        let parsed = parse(r#"fix @"docs/path with spaces.md" and @src/a\ b.rs now"#);

        assert_eq!(parsed.clean_text, "fix and now");
        assert_eq!(parsed.mentions.len(), 2);
        assert_eq!(parsed.mentions[0].raw, "docs/path with spaces.md");
        assert_eq!(parsed.mentions[1].raw, "src/a b.rs");
    }

    #[test]
    fn classifies_special_mentions() {
        let parsed = parse("@tx:latest @chat:demo @memory:auth");
        assert_eq!(parsed.mentions[0].kind, MentionKind::Transaction);
        assert_eq!(parsed.mentions[1].kind, MentionKind::Chat);
        assert_eq!(parsed.mentions[2].kind, MentionKind::Memory);
    }
}
