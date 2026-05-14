use std::fs;

use anyhow::Result;

use super::*;

#[test]
fn detects_nextjs_routes_components_and_exports() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let page = dir.path().join("src/app/courses/page.tsx");
    let component = dir.path().join("src/components/CourseCard.tsx");
    fs::create_dir_all(page.parent().unwrap())?;
    fs::create_dir_all(component.parent().unwrap())?;
    fs::write(&page, "export function CoursesPage() { return null }\n")?;
    fs::write(&component, "export const CourseCard = () => null\n")?;

    let maps = build(dir.path())?;

    assert!(maps.routes.iter().any(|route| route.route == "/courses"));
    assert!(maps
        .components
        .iter()
        .any(|component| component.name == "CourseCard"));
    assert!(maps
        .exports
        .iter()
        .any(|export| export.symbol == "CourseCard"));
    Ok(())
}

#[test]
fn detects_stale_map_entries_by_hash() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let page = dir.path().join("src/app/courses/page.tsx");
    fs::create_dir_all(page.parent().unwrap())?;
    fs::write(&page, "export function CoursesPage() { return null }\n")?;
    write(dir.path())?;
    fs::write(&page, "export function CoursesPage() { return 'new' }\n")?;

    let validation = validate_existing(dir.path())?;

    assert!(validation.stale);
    assert!(validation
        .stale_entries
        .iter()
        .any(|entry| entry.file == "src/app/courses/page.tsx"));
    Ok(())
}

#[test]
fn selects_map_context_without_source_contents() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let page = dir.path().join("src/app/courses/page.tsx");
    fs::create_dir_all(page.parent().unwrap())?;
    fs::write(
        &page,
        "SECRET_SOURCE_BODY\nexport function CoursesPage() {}\n",
    )?;
    write(dir.path())?;

    let spec = fixture_spec();
    let selected = select_context(dir.path(), &spec)?;
    let serialized = serde_json::to_string(&selected)?;

    assert!(selected
        .routes
        .iter()
        .any(|route| route.route == "/courses"));
    assert!(!serialized.contains("SECRET_SOURCE_BODY"));
    Ok(())
}

fn fixture_spec() -> crate::spec::AgentSpec {
    crate::spec::AgentSpec {
        task: crate::spec::TaskSpec {
            id: "courses".to_string(),
            kind: "code.command".to_string(),
            title: None,
            target: Some("/courses".to_string()),
        },
        agent: crate::spec::AgentConfig::default(),
        agents: crate::spec::RoleAgents::default(),
        topology: crate::spec::TopologySpec::default(),
        workspace: crate::spec::WorkspaceSpec {
            kind: "code.git".to_string(),
            isolation: Some("git_worktree".to_string()),
            root: None,
        },
        skills: Vec::new(),
        execution: crate::spec::ExecutionSpec::default(),
        scope: crate::spec::ScopeSpec {
            allow: vec!["src/app/**".to_string()],
            deny: Vec::new(),
        },
        rules: Vec::new(),
        verify: crate::spec::VerifySpec::default(),
        review: crate::spec::ReviewSpec::default(),
        repair: crate::spec::RepairSpec::default(),
        transaction: crate::spec::TransactionSpec::default(),
    }
}
