use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Color {
    Green,
    Red,
    Yellow,
    Blue,
    Cyan,
    Magenta,
    Gray,
    DarkGray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StepStatus {
    Done,
    Running,
    Failed,
    Pending,
}

pub(super) fn success(message: &str) {
    println!("  {}[ok]{} {}", color(Color::Green), reset(), message);
}

pub(super) fn warning(message: &str) {
    println!("  {}[warn]{} {}", color(Color::Yellow), reset(), message);
}

pub(super) fn error(message: &str) {
    println!("  {}[error]{} {}", color(Color::Red), reset(), message);
}

pub(super) fn info(message: &str) {
    println!("  {}{}{}", color(Color::Blue), message, reset());
}

pub(super) fn section(title: &str) {
    println!();
    println!("  {}{}{}", bold_color(Color::Cyan), title, reset());
}

pub(super) fn muted(message: &str) -> String {
    format!("{}{}{}", color(Color::DarkGray), message, reset())
}

pub(super) fn status_label(status: &str) -> String {
    let tint = match status {
        "COMMITTED" | "CLOSED" | "DONE" | "ok" | "ready" | "low" | "pinned" => Color::Green,
        "FAILED" | "ERROR" | "ROLLED_BACK" | "missing" | "high" => Color::Red,
        "BLOCKED_ON_HUMAN" | "CANCELED" | "approval_required" | "medium" => Color::Yellow,
        _ => Color::Gray,
    };
    format!("{}{}{}", color(tint), status, reset())
}

pub(super) fn progress_step(elapsed: &str, status: StepStatus, message: &str) -> String {
    let (label, tint) = match status {
        StepStatus::Done => ("ok", Color::Green),
        StepStatus::Running => ("run", Color::Yellow),
        StepStatus::Failed => ("fail", Color::Red),
        StepStatus::Pending => ("wait", Color::DarkGray),
    };
    format!(
        "  {}[{}]{} {}[{}]{} {}",
        color(Color::DarkGray),
        elapsed,
        reset(),
        color(tint),
        label,
        reset(),
        message
    )
}

pub(super) fn suggestions(items: &[String]) {
    if items.is_empty() {
        return;
    }
    println!();
    println!("  {}What next?{}", color(Color::DarkGray), reset());
    for chunk in items.chunks(2) {
        if let [left, right] = chunk {
            println!("  - {:<28} - {}", quote(left), quote(right));
        } else {
            println!("  - {}", quote(&chunk[0]));
        }
    }
}

pub(super) fn diff_from_str(diff: &str) -> String {
    let mut out = String::new();
    for line in diff.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            let _ = writeln!(&mut out, "{}{}{}", color(Color::Green), line, reset());
        } else if line.starts_with('-') && !line.starts_with("---") {
            let _ = writeln!(&mut out, "{}{}{}", color(Color::Red), line, reset());
        } else {
            let _ = writeln!(&mut out, "{line}");
        }
    }
    out
}

pub(super) fn code_block(language: &str, code: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(
        &mut out,
        "{}```{}{}",
        color(Color::DarkGray),
        language,
        reset()
    );
    for line in code.lines() {
        let _ = writeln!(&mut out, "{}", highlight_line(line));
    }
    let _ = writeln!(&mut out, "{}```{}", color(Color::DarkGray), reset());
    out
}

fn highlight_line(line: &str) -> String {
    let mut output = String::new();
    for part in line.split_inclusive(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        let token = part.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_');
        if is_keyword(token) {
            output.push_str(&part.replacen(token, &styled(token, Color::Magenta), 1));
        } else if is_type(token) {
            output.push_str(&part.replacen(token, &styled(token, Color::Cyan), 1));
        } else {
            output.push_str(part);
        }
    }
    output
}

fn is_keyword(token: &str) -> bool {
    matches!(
        token,
        "fn" | "let"
            | "mut"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "if"
            | "else"
            | "match"
            | "return"
            | "async"
            | "await"
    )
}

fn is_type(token: &str) -> bool {
    matches!(
        token,
        "String" | "Vec" | "Option" | "Result" | "bool" | "i32" | "u64" | "f64" | "str" | "char"
    )
}

fn quote(value: &str) -> String {
    format!("\"{value}\"")
}

pub(super) fn styled(value: &str, tint: Color) -> String {
    format!("{}{}{}", color(tint), value, reset())
}

pub(super) fn color(color: Color) -> &'static str {
    if no_color() {
        return "";
    }
    match color {
        Color::Green => "\x1b[32m",
        Color::Red => "\x1b[31m",
        Color::Yellow => "\x1b[33m",
        Color::Blue => "\x1b[34m",
        Color::Cyan => "\x1b[36m",
        Color::Magenta => "\x1b[35m",
        Color::Gray => "\x1b[37m",
        Color::DarkGray => "\x1b[90m",
    }
}

pub(super) fn bold_color(color_value: Color) -> String {
    if no_color() {
        return String::new();
    }
    format!("\x1b[1m{}", color(color_value))
}

pub(super) fn reset() -> &'static str {
    if no_color() {
        ""
    } else {
        "\x1b[0m"
    }
}

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_diff_lines() {
        let output = diff_from_str("- old\n+ new\n context");
        assert!(output.contains("- old"));
        assert!(output.contains("+ new"));
        assert!(output.contains("context"));
    }

    #[test]
    fn formats_progress_steps() {
        let output = progress_step("00:01", StepStatus::Done, "build passed");
        assert!(output.contains("[00:01]"));
        assert!(output.contains("[ok]"));
    }
}
