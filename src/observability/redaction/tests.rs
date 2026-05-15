use std::fs;

use anyhow::Result;

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

#[test]
fn redacts_secret_object_keys_even_with_short_values() -> Result<()> {
    let value = serde_json::json!({
        "api_key": "abc",
        "nested": { "password": "pw" },
        "safe": "visible"
    });

    let (redacted, findings) = redact_value_with_findings(&value)?;

    assert_eq!(redacted["api_key"], "<redacted>");
    assert_eq!(redacted["nested"]["password"], "<redacted>");
    assert_eq!(redacted["safe"], "visible");
    assert_eq!(findings.len(), 2);
    Ok(())
}

#[test]
fn redacts_file_in_place() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("log.txt");
    fs::write(&path, "Bearer secret.jwt.value")?;

    let findings = redact_file_in_place(&path)?;

    let text = fs::read_to_string(&path)?;
    assert!(!text.contains("secret.jwt.value"));
    assert!(!findings.is_empty());
    Ok(())
}
