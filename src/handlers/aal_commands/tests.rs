use super::*;

#[test]
fn aal_check_matches_expected_artifacts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let input = dir.path().join("demo.aal");
    fs::write(
        &input,
        r#"
change Demo {
  workspace code.git
  goal "Demo"
  allow edit:
    - "src/**"
}
"#,
    )?;
    let expected = dir.path().join("expected");
    fs::create_dir_all(&expected)?;
    let parsed = aal::parse_aal_file(&input)?;
    let dag = compiler::compile(&parsed.spec)?;
    fs::write(
        expected.join("demo.yaml"),
        serde_yaml::to_string(&parsed.spec)?,
    )?;
    fs::write(expected.join("demo.ir"), parsed.spec.to_agent_ir())?;
    fs::write(
        expected.join("demo.dag.json"),
        serde_json::to_string_pretty(&dag)?,
    )?;

    let result = check_artifacts(&input, Some(&expected), false)?;
    assert_eq!(result.task_id, "demo");
    assert_eq!(result.expected_dir.as_deref(), Some(expected.as_path()));
    Ok(())
}

#[test]
fn aal_check_accepts_crlf_golden_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("golden.txt");
    fs::write(&path, "a\r\nb\r\n")?;
    compare_file(&path, "a\nb\n")?;
    Ok(())
}

#[test]
fn aal_format_check_detects_unformatted_input() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let input = dir.path().join("demo.aal");
    fs::write(&input, "change Demo {\nworkspace code.git\n}\n")?;

    let error = format(&input, None, true).unwrap_err();

    assert!(error.to_string().contains("AAL format mismatch"));
    Ok(())
}
