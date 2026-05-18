use std::fs;

use anyhow::Result;

use super::{
    dashboard_text, render_dashboard, ApprovalPanel, ComposerPanel, Dashboard, DashboardSummary,
    EventRailItem, MemoryPanel, ProviderPanel, ProviderRoleLine, ProviderStatusLine, ShellPanel,
    ShellStatusLine, SlashPaletteItem, ToolCard, TransactionSummary, TranscriptLine,
};
use crate::agent_dir::init_project;
use crate::product_cli::{config, providers};

#[test]
fn renders_terminal_dashboard_panels() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    let tx = dir.path().join(".agent/tx/tx-20260101000000-demo");
    fs::create_dir_all(&tx)?;
    fs::write(
        tx.join("journal.jsonl"),
        "{\"ts\":\"2026-01-01T00:00:00Z\",\"tx_id\":\"tx-20260101000000-demo\",\"state\":\"CREATED\",\"message\":\"created\",\"data\":{}}\n{\"ts\":\"2026-01-01T00:00:01Z\",\"tx_id\":\"tx-20260101000000-demo\",\"state\":\"COMMITTED\",\"message\":\"committed\",\"data\":{}}\n",
    )?;
    fs::write(
        tx.join("dag.json"),
        r#"{"nodes":[{"id":"executor"}],"edges":[]}"#,
    )?;
    fs::write(tx.join("verifier.json"), r#"{"passed":true}"#)?;
    fs::write(
        tx.join("verifier.log"),
        "COMMAND: cargo test\nSUCCESS: true\n",
    )?;
    fs::write(
        tx.join("cost.json"),
        r#"{"total_usd":0.01,"estimated_tokens":42}"#,
    )?;
    fs::write(
        tx.join("effects.jsonl"),
        "{\"effect_id\":\"eff-1\",\"status\":\"verified\"}\n",
    )?;
    fs::write(
        tx.join("heartbeat.jsonl"),
        "{\"event\":\"HEARTBEAT\",\"node\":\"executor\",\"last_output_sec\":4}\n",
    )?;
    fs::write(
        tx.join("agent_trace.json"),
        r#"{"routes":{"executor":{"selected_adapter":"deepseek"}}}"#,
    )?;
    fs::write(
        tx.join("tool_loop_executor.json"),
        r#"{
          "status":"ok",
          "plan_source":"native_tool_call",
          "blocked":false,
          "blocked_reason":null,
          "native_tool_calls":[{"id":"plan-1","name":"agenthub_command_plan"}],
          "command_permissions":[{"tool":"shell","action":"npm run build","profile":"workspace-write","approval_required":false,"risk":"medium","reason":"test"}]
        }"#,
    )?;
    fs::write(
        tx.join("tool_results_executor.json"),
        r#"{
          "status":"ok",
          "blocked":false,
          "blocked_reason":null,
          "policy_summary":{"max_tool_rounds":3,"rounds_used":1,"total_results":1,"approval_required_results":0,"truncated_results":0,"protected_path_results":0,"binary_skipped_results":0,"symlink_denied_results":0,"network_denied_results":0},
          "rounds":[{"round":1,"results":[{"name":"read_file","status":"ok"}]}]
        }"#,
    )?;
    fs::create_dir_all(tx.join("logs"))?;
    fs::write(
        tx.join("logs/execution-0.stdout.log"),
        "line one\nline two\n",
    )?;
    let chats = dir.path().join(".agent/shell/chats");
    fs::create_dir_all(&chats)?;
    fs::write(
        chats.join("chat-demo.jsonl"),
        "{\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"user_message\",\"text\":\"check server load\"}\n\
         {\"at\":\"2026-01-01T00:00:01Z\",\"kind\":\"intent_classified\",\"intent\":\"ops_advice\",\"mode\":\"ops\",\"reason\":\"server wording\",\"text\":\"check server load\"}\n\
         {\"at\":\"2026-01-01T00:00:02Z\",\"kind\":\"context_built\",\"text\":\"context built\",\"prompt_tokens\":64,\"max_prompt_tokens\":6000,\"memory_tokens\":12,\"context_compressed\":true}\n\
         {\"at\":\"2026-01-01T00:00:03Z\",\"kind\":\"provider_requested\",\"provider\":\"deepseek\",\"model\":\"deepseek-chat\",\"request_id\":\"chat-1\",\"prompt_tokens\":64,\"text\":\"deepseek request started\"}\n\
         {\"at\":\"2026-01-01T00:00:04Z\",\"kind\":\"assistant_delta\",\"provider\":\"deepseek\",\"text\":\"stream chunk\"}\n\
         {\"at\":\"2026-01-01T00:00:05Z\",\"kind\":\"assistant_message\",\"provider\":\"deepseek\",\"text\":\"Load is normal\"}\n\
         {\"at\":\"2026-01-01T00:00:06Z\",\"kind\":\"turn_finished\",\"provider\":\"deepseek\",\"status\":\"succeeded\",\"prompt_tokens\":64,\"completion_tokens\":5,\"total_tokens\":69,\"estimated_cost_usd\":0.00001,\"text\":\"turn succeeded\"}\n\
         {\"at\":\"2026-01-01T00:00:07Z\",\"kind\":\"memory_extraction\",\"mode\":\"ops\",\"domain\":\"ops\",\"candidates_added\":2,\"text\":\"memory extraction added 2 inbox candidate(s)\"}\n",
    )?;
    fs::create_dir_all(dir.path().join(".agent/specs"))?;
    fs::write(
        dir.path().join(".agent/specs/approval.yaml"),
        "transaction:\n  approval_required: true\n",
    )?;
    fs::write(dir.path().join(".deepseek"), "test-key\n")?;
    config::set_value(dir.path(), "default_provider", "deepseek")?;
    providers::set_role_provider(dir.path(), "executor", "deepseek")?;
    providers::set_role_fallback(
        dir.path(),
        "reviewer",
        &["deepseek".to_string(), "kimi".to_string()],
    )?;

    let dashboard = dashboard_text(dir.path())?;

    assert!(dashboard.contains("AgentHub TUI Dashboard"));
    assert!(dashboard.contains("[Status Line]"));
    assert!(dashboard.contains("- mode: project | provider: deepseek"));
    assert!(dashboard.contains("- chat: chat-demo check server load"));
    assert!(dashboard.contains("- tokens: prompt 64 total 69 | cost: 0.000010 USD"));
    assert!(dashboard.contains("- controls: Ctrl-C interrupt | /resume | /messages | /context"));
    assert!(dashboard.contains("[Composer]"));
    assert!(dashboard.contains("/messages"));
    assert!(dashboard.contains("@tx:latest"));
    assert!(dashboard.contains("@chat:chat-demo"));
    assert!(dashboard.contains("[Chat Transcript]"));
    assert!(dashboard.contains("user: check server load"));
    assert!(dashboard.contains("assistant stream: stream chunk"));
    assert!(dashboard.contains("[Event Rail]"));
    assert!(dashboard.contains("[memory] memory extraction"));
    assert!(dashboard.contains("[streaming] assistant delta"));
    assert!(dashboard.contains("[ready] context built: prompt 64/6000 memory 12 compressed true"));
    assert!(dashboard.contains("[Live Tool Cards]"));
    assert!(dashboard.contains("[memory] memory: memory extraction"));
    assert!(dashboard.contains("[done] cost: deepseek turn succeeded"));
    assert!(dashboard.contains("tokens prompt 64 completion 5 total 69 cost 0.000010 USD"));
    assert!(dashboard.contains("[done] command_plan: tx-20260101000000-demo executor command plan"));
    assert!(dashboard.contains("native_calls 1 commands 1 approvals 0"));
    assert!(dashboard.contains("[done] tool_results: tx-20260101000000-demo executor tool results"));
    assert!(dashboard
        .contains("rounds 1 results 1 approvals 0 protected 0 truncated 0 network_denied 0"));
    assert!(dashboard.contains("tool_results_executor.json"));
    assert!(dashboard.contains("[Summary]"));
    assert!(dashboard.contains("- total transactions: 1"));
    assert!(dashboard.contains("- committed: 1 | rolled back: 0 | blocked: 0 | running: 0"));
    assert!(dashboard.contains("[Transactions]"));
    assert!(dashboard.contains("tx-20260101000000-demo COMMITTED"));
    assert!(dashboard.contains("- stage: COMMITTED"));
    assert!(dashboard.contains("- provider: deepseek"));
    assert!(dashboard.contains("- effects: 1"));
    assert!(dashboard.contains("- heartbeat: executor, last output 4s ago"));
    assert!(dashboard.contains("line two"));
    assert!(dashboard.contains("[Providers]"));
    assert!(dashboard.contains("- default: deepseek"));
    assert!(dashboard.contains("- executor -> deepseek (ok)"));
    assert!(dashboard.contains("- reviewer -> deepseek (ok) fallback:deepseek,kimi"));
    assert!(dashboard.contains("- DAG: 1 nodes, 0 edges"));
    assert!(dashboard.contains("- pending specs: 1"));
    assert!(dashboard.contains("[Next Actions]"));
    assert!(dashboard.contains("agenthub tx report tx-20260101000000-demo"));
    Ok(())
}

#[test]
fn renders_shell_tui_regression_snapshot() {
    let dashboard = Dashboard {
        project: "/workspace/site2".to_string(),
        shell: ShellPanel {
            status: ShellStatusLine {
                mode: "ops".to_string(),
                provider: "deepseek".to_string(),
                provider_ready: true,
                model: Some("deepseek-chat".to_string()),
                git_state: "git optional".to_string(),
                agent_state: "global session".to_string(),
                chat_id: Some("chat-shell-smoke".to_string()),
                chat_title: Some("Ops smoke".to_string()),
                prompt_tokens: Some(12),
                total_tokens: Some(18),
                estimated_cost_usd: Some(0.0000042),
                controls: vec![
                    "Ctrl-C interrupt".to_string(),
                    "/resume".to_string(),
                    "/messages".to_string(),
                    "/context".to_string(),
                ],
            },
            composer: ComposerPanel {
                prompt: "Type a request, / command, @ context, ! shell command, or # memory note"
                    .to_string(),
                slash_palette: vec![
                    SlashPaletteItem {
                        command: "/status".to_string(),
                        summary: "show mode, provider, git, and current tx".to_string(),
                    },
                    SlashPaletteItem {
                        command: "/providers".to_string(),
                        summary: "inspect DeepSeek/Kimi API setup".to_string(),
                    },
                ],
                context_mentions: vec![
                    "@file".to_string(),
                    "@tx:latest".to_string(),
                    "@chat:chat-shell-smoke".to_string(),
                ],
            },
            transcript: vec![
                TranscriptLine {
                    at: "2026-05-18T01:00:00Z".to_string(),
                    speaker: "user".to_string(),
                    text: "/mode ops".to_string(),
                },
                TranscriptLine {
                    at: "2026-05-18T01:00:01Z".to_string(),
                    speaker: "tool".to_string(),
                    text: "shell classified as read-only (approval not required, risk: low)"
                        .to_string(),
                },
            ],
            event_rail: vec![
                EventRailItem {
                    at: "2026-05-18T01:00:02Z".to_string(),
                    state: "ready".to_string(),
                    label: "tool permission".to_string(),
                    detail: "read-only risk low approval false".to_string(),
                },
                EventRailItem {
                    at: "2026-05-18T01:00:03Z".to_string(),
                    state: "done".to_string(),
                    label: "turn finished".to_string(),
                    detail: "deepseek succeeded turn succeeded".to_string(),
                },
            ],
            tool_cards: vec![
                ToolCard {
                    kind: "tool_permission".to_string(),
                    state: "ready".to_string(),
                    title: "shell read-only".to_string(),
                    detail: "risk low approval false action printf shell-smoke-ok".to_string(),
                    link: None,
                },
                ToolCard {
                    kind: "cost".to_string(),
                    state: "done".to_string(),
                    title: "deepseek turn succeeded".to_string(),
                    detail: "tokens prompt 12 completion 6 total 18 cost 0.000004 USD".to_string(),
                    link: None,
                },
            ],
        },
        summary: DashboardSummary {
            total: 0,
            committed: 0,
            rolled_back: 0,
            blocked: 0,
            running: 0,
        },
        transactions: vec![TransactionSummary {
            id: "tx-shell-smoke".to_string(),
            status: "NOOP".to_string(),
        }],
        latest: None,
        providers: ProviderPanel {
            default_provider: "deepseek".to_string(),
            ready: 1,
            missing: 1,
            profiles: 2,
            statuses: vec![
                ProviderStatusLine {
                    id: "deepseek".to_string(),
                    state: "ok".to_string(),
                    is_default: true,
                    detail: "https://api.deepseek.com/v1".to_string(),
                    model: Some("deepseek-chat".to_string()),
                },
                ProviderStatusLine {
                    id: "kimi".to_string(),
                    state: "blocked".to_string(),
                    is_default: false,
                    detail: "external credential".to_string(),
                    model: Some("kimi-k2.6".to_string()),
                },
            ],
            roles: vec![ProviderRoleLine {
                role: "executor".to_string(),
                provider: "deepseek".to_string(),
                available: Some(true),
                fallback: vec!["deepseek".to_string(), "kimi".to_string()],
            }],
        },
        memory: MemoryPanel {
            committed: 0,
            failed_attempts: 0,
            recent_changes: 0,
        },
        approvals: ApprovalPanel {
            specs: Vec::new(),
            blocked_transactions: Vec::new(),
        },
        next_actions: vec!["agenthub run \"describe the change\" --no-commit".to_string()],
    };

    let expected = "\
AgentHub TUI Dashboard
Project: /workspace/site2
Tabs: Chat | Events | Run | Transactions | Diff | Logs | Effects | Approvals | Memory | Providers

[Status Line]
- mode: ops | provider: deepseek ok model:deepseek-chat | git optional | global session
- chat: chat-shell-smoke Ops smoke
- tokens: prompt 12 total 18 | cost: 0.000004 USD
- controls: Ctrl-C interrupt | /resume | /messages | /context

[Composer]
- prompt: Type a request, / command, @ context, ! shell command, or # memory note
- slash palette:
  - /status      show mode, provider, git, and current tx
  - /providers   inspect DeepSeek/Kimi API setup
- context mentions: @file @tx:latest @chat:chat-shell-smoke

[Chat Transcript]
- 01:00:00 user: /mode ops
- 01:00:01 tool: shell classified as read-only (approval not required, risk: low)

[Event Rail]
- 01:00:02 [ready] tool permission: read-only risk low approval false
- 01:00:03 [done] turn finished: deepseek succeeded turn succeeded

[Live Tool Cards]
- [ready] tool_permission: shell read-only
  risk low approval false action printf shell-smoke-ok
- [done] cost: deepseek turn succeeded
  tokens prompt 12 completion 6 total 18 cost 0.000004 USD

[Summary]
- total transactions: 0
- committed: 0 | rolled back: 0 | blocked: 0 | running: 0

[Transactions]
- tx-shell-smoke NOOP

[Latest Transaction]
- none

[Providers]
- default: deepseek
- ready: 1 | missing: 1 | profiles: 2
- deepseek [ok default] https://api.deepseek.com/v1 model:deepseek-chat
- kimi [blocked] external credential model:kimi-k2.6
- roles:
  - executor -> deepseek (ok) fallback:deepseek,kimi

[Memory]
- committed records: 0
- failed attempts: 0
- recent workspace changes: 0

[Approvals]
- pending specs: 0
- blocked transactions: 0

[Next Actions]
- agenthub run \"describe the change\" --no-commit
";

    assert_eq!(render_dashboard(&dashboard), expected);
}
