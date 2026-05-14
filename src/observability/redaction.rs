use anyhow::Result;
use regex::Regex;

pub fn redact_text(input: &str) -> Result<String> {
    let replacements = [
        (
            r#"(?i)(api[_-]?key|token|password|secret|database_url|db_url)\s*[:=]\s*['"]?[^'"\s]+"#,
            "$1=<redacted>",
        ),
        (r#"(?i)bearer\s+[A-Za-z0-9._\-]+"#, "Bearer <redacted>"),
        (r#"sk-[A-Za-z0-9_\-]{10,}"#, "sk-<redacted>"),
        (
            r#"(?i)(postgres|postgresql|mysql|mongodb|redis)://[^'"\s]+"#,
            "$1://<redacted>",
        ),
    ];

    let mut output = input.to_string();
    for (pattern, replacement) in replacements {
        let regex = Regex::new(pattern)?;
        output = regex.replace_all(&output, replacement).to_string();
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_common_secret_shapes() -> Result<()> {
        let text = "token=abcd1234 Bearer secret.jwt.value postgres://user:pass@localhost/db sk-1234567890abcdef";
        let redacted = redact_text(text)?;

        assert!(!redacted.contains("abcd1234"));
        assert!(!redacted.contains("secret.jwt.value"));
        assert!(!redacted.contains("user:pass"));
        assert!(!redacted.contains("1234567890abcdef"));
        assert!(redacted.contains("<redacted>"));
        Ok(())
    }
}
