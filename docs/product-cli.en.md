# Product CLI

Languages: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

AgentHub PRD v3 adds product-facing commands for local installation checks, provider readiness, simple configuration, and chat-first local work.

## Chat-First Shell

```bash
agenthub
```

Running `agenthub` without a subcommand is the recommended daily entry. In an uninitialized folder it starts Chat Mode without creating Git or `.agent`; project bootstrap is deferred until a project transaction really needs it. The shell restores the latest chat, shows provider readiness, and lets you type a normal task. File-changing tasks still create a draft plan, show scope, patch preview, verifier plan, protected-path warnings, and rollback receipts, ask for inline approval, run the transaction, and then suggest `/diff`, `/logs`, `/report`, `/explain`, and `/undo`.

AgentHub records and displays Chat/Ops/Project mode decisions. Plain chat stays in Chat Mode, server or operations wording without a project runtime is marked as Ops Mode, and initialized `.agent` workspaces are Project Mode. Prompt chips, `/context`, `/status`, and headless `exec --jsonl` expose the selected mode.

Draft-only flows stay lazy too: `plan`, `ask`, and shell draft creation store drafts under the AgentHub user data directory until a project runtime exists. Git initialization, `.agent` creation, and baseline commit are planned and confirmed only when a transaction is about to run; non-interactive automation keeps the existing auto-bootstrap behavior.

Explicit `!command` shell actions now receive an AgentHub-owned tool permission decision before execution. The transcript records `tool_permission` events with profile (`chat`, `read-only`, `workspace-write`, `ops-host`), risk, `approval_required`, and reason; high-risk local destructive commands, package changes, mutating HTTP calls, and mutating Ops host/container/cluster commands ask for approval before running.

For Ops Mode, explicit shell commands also write host-scoped receipts. AgentHub extracts targets from commands such as `ssh`, `scp`, `rsync`, `kubectl`, `helm`, `systemctl`, `journalctl`, `docker`, and `terraform`, stores host profiles under the AgentHub user data directory, records trust metadata, and links matching runbook cards. Hosts marked `untrusted` require approval even for otherwise read-only Ops commands.

Use `/` for commands, `/cd <folder>` to switch projects without restarting, `@path` for context, `!command` for policy-checked shell commands, and `# note` for memory. In Chat/Ops Mode, memory is stored under the AgentHub user data directory; initialized projects keep project memory under `.agent/memory`.

Chat sessions are restored automatically. Use `/chats` to list sessions with auto titles and pin state, `/search <text>` to search titles/messages, `/rename <title>` to name the current chat, and `/pin` or `/unpin` to keep important work at the top. If a chat JSONL transcript contains a corrupt line, AgentHub keeps the valid events and exposes a `session_recovery` event instead of dropping the whole transcript.

Use `/context` before a task to preview the current chat title, recent messages, memory summary, selected transaction report, and supported mention forms.

Use `agenthub tui` or `agenthub tui --live` for an event-backed terminal surface with status line, composer hints, slash palette, `@` context mentions, latest chat transcript, a live event rail for provider/streaming/tool/turn state, and live tool cards for tool permissions, approval stops, memory extraction, cost/tokens, command-plan receipts, and tool-result reinjection receipts.

## Headless Exec

```bash
agenthub exec "answer with one word: ok"
agenthub exec "answer with one word: ok" --jsonl
```

`exec` runs one API-native chat turn through the same DeepSeek/Kimi provider selection and AgentHub-owned chat event store. It does not initialize Git or `.agent` for a plain chat request. The provider prompt includes budgeted relevant committed memory, but pending memory inbox candidates stay out of context until approval. With `--jsonl`, it prints the live session event stream, including `intent_classified`, `context_built`, `provider_requested`, `assistant_delta`, `provider_finished`, `turn_finished`, and `memory_extraction`; `context_built` includes memory token counts, prompt budget, expired/conflict/budget-dropped records, recent-message drops, and whether context was compressed. Completed provider and turn events include token counts, estimated USD cost, and pricing source. The post-turn `memory_extraction` event reports whether AgentHub added review-only inbox candidates or skipped extraction.

In an initialized project, `exec` treats a file-changing request like the interactive shell: it writes an approval-required draft, emits `draft_created`, `approval_required`, and `turn_finished status=approval_required` JSONL events, and exits with code `2` instead of running without approval.

DeepSeek/Kimi project execution can first run bounded AgentHub-owned `read_file`, `list_dir`, `search`, and read-only `shell` tools, records redacted `tool_results_<role>.json` receipts with path/output/network/limit policy summaries, reinjects those results into the same provider turn, then requests a native `agenthub_command_plan` tool call or JSON fallback and permission-checks proposed commands before execution.

## Headless Ops Exec

```bash
agenthub ops exec "uptime"
agenthub ops exec "uptime" --jsonl
```

`ops exec` is the non-interactive path for DevOps-style shell checks. It uses the same AgentHub-owned tool permission and command policy classifiers as interactive `!command`, writes command logs under the AgentHub user data directory, updates host profiles, and records host-scoped Ops receipts. It does not create `.agent` in an empty folder. Commands that require approval are recorded as approval-required receipts and are not executed by the headless path.

## Chat Usage Stats

```bash
agenthub stats
```

`stats` summarizes stored chat `turn_finished` events for the current project scope, including turn count, prompt tokens, completion tokens, total tokens, estimated USD cost, and provider-level totals. Inside the interactive shell, use `/stats` for the same view.

## Doctor

```bash
agenthub doctor
```

`doctor` is the first readiness screen after install. It checks the AgentHub version, binary path, dev/release channel, OS/architecture, `sh` shell availability, Git version, Git repository status, `.agent` initialization, policy files, default provider readiness, and supported provider binaries/endpoints. Missing optional DeepSeek/Kimi/Kimi APIs are warnings; missing Git or `sh` is blocking.

## Version

```bash
agenthub version
```

Prints the installed AgentHub version.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style" --no-watch
agenthub run examples/command-task.yaml
```

`plan` writes a draft AgentSpec under `.agent/drafts/` unless `--output` is provided. `run` accepts either an existing AgentSpec path or a natural request. Natural requests are converted into a draft spec first, then executed through the normal transaction engine.

In an interactive terminal, `run` prints live journal progress while the transaction executes. Use `--no-watch` for quiet one-shot runs. Non-TTY/scripted output keeps the compact `tx-id STATUS (report)` line, followed by task, provider, topology, verifier, memory promotion, changed file count, report, `tx explain`, `tx watch`, and dashboard path.

```bash
agenthub tx explain tx-20260515123000-abcd1234
agenthub tx diff tx-20260515123000-abcd1234
agenthub tx logs tx-20260515123000-abcd1234 --tail 80
```

`tx explain` summarizes why a transaction failed or succeeded, what happened, what to do next, and which artifacts to inspect.
`tx diff` shows the committed patch when available and falls back to diff-guard summaries for uncommitted transactions.
`tx logs` prints bounded command logs, optionally filtered by stage and tail length.

Transaction commands that target one transaction accept either an explicit id or `latest`/`last`. This applies to `tx report`, `tx effects`, `tx explain`, `tx diff`, `tx logs`, `tx watch`, `tx cancel`, `tx resolve`, `tx resume`, and `tx retry`.

## Undo

```bash
agenthub undo last
agenthub undo tx-20260515123000-abcd1234
```

`undo` creates a normal Git revert for a committed AgentHub transaction. It refuses to run when the working tree has unrelated uncommitted changes and records `.agent/tx/<tx-id>/undo.json`.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup deepseek
agenthub providers setup kimi
DEEPSEEK_API_KEY=... agenthub providers test deepseek
KIMI_API_KEY=... agenthub providers test kimi
agenthub providers diagnose deepseek
agenthub providers recovery --json
agenthub providers unblock kimi
agenthub providers inspect-key kimi
agenthub providers inspect-key kimi --json
agenthub providers inspect-key kimi --from-file ./new-kimi.key
agenthub providers preflight-key kimi --from-file ./new-kimi.key
agenthub providers rc-unblock kimi --from-file ./new-kimi.key
agenthub providers rotate-key kimi --from-file ./new-kimi.key
scripts/kimi-rc-unblock.sh
agenthub providers set executor deepseek
agenthub providers fallback chat deepseek kimi
agenthub providers fallback reviewer deepseek kimi
```

Inside the interactive shell, `/providers` opens a provider wizard with API readiness, default markers, role assignments, fallbacks, and the next setup/diagnose/test commands.

`providers status --json` is the raw machine-readable provider state. Each row includes a readiness `check_id`, and blocked or missing DeepSeek/Kimi rows include `blocker_kind: "external_credential"` plus `next_commands` so automation can move from raw state to safe recovery commands without parsing `detail`. Kimi rows can also include redacted `credential_classification` values such as `kimi_code_cli_oauth` or `kimi_code_cli_oauth_reported`. When a Kimi row matches a blocked auth report, the same JSON row also includes redacted `auth_status`, `auth_key_sha256_12`, `auth_key_source`, `credential_warning`, and `next_action` fields.

Supported providers:

- `deepseek`: DeepSeek OpenAI-compatible API endpoint. Defaults to `https://api.deepseek.com/v1`; reads `DEEPSEEK_API_KEY`, with `ANTHROPIC_AUTH_TOKEN` accepted for DeepSeek-compatible deployments.
- `kimi`: Kimi/Moonshot API endpoint. Defaults to `https://api.moonshot.ai/v1`; reads `KIMI_API_KEY` or `MOONSHOT_API_KEY`.

The local command runner is internal to the transaction kernel; it is not a user-facing AI provider.

AgentHub also reads key files named `.deepseek` and `.kimi` from the project directory, current shell directory, or their parent directories. `DEEPSEEK_API_KEY_FILE`, `ANTHROPIC_AUTH_TOKEN_FILE`, `KIMI_API_KEY_FILE`, and `MOONSHOT_API_KEY_FILE` can point at explicit key files.

`setup` configures a provider only when it is available. On success it records `default_provider`, prints the endpoint, reports the dry-run mode, and shows the next `agenthub ask` command.

Example:

```text
configured	deepseek
default_provider	deepseek
endpoint	https://api.deepseek.com/v1
dry_run	API request test is performed by providers test
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers diagnose <id>` prints endpoint, model, API-key marker, safe key source/length/fingerprint metadata, auth hint, status hint, install hint, scheme, and provider-specific details. It never prints secret values.

`providers recovery --json` is the first machine-readable recovery entrypoint. It summarizes provider state, `blocker_scope`, `blocker_kinds`, top-level `blocked_checks`, per-provider actions, and the readiness gate commands without printing keys.

`providers set <role> <provider>` stores `provider.role.<role>` in `.agent/config.yaml`. `providers fallback <role> ...` stores a comma-separated fallback chain under `provider.fallback.<role>`. Valid roles are planner, executor, reviewer, repair, generator, critic, researcher, aggregator, chat, manager, and worker. Chat turns use `provider.role.chat` plus `provider.fallback.chat` before falling back to any other available API provider.

Named HTTP profiles are intentionally disabled in API-native mode. Provider logs, retries, memory, and future tool calls stay inside AgentHub for the two supported APIs.

`providers test deepseek` and `providers test kimi` perform real OpenAI-compatible completion requests and then try optional `/v1/models`; a missing models endpoint is reported as `models unavailable`, not as a failed provider test. If the completion request fails with auth, rate-limit, timeout, transport, or server errors, the command prints a structured failure receipt with `request_id`, endpoint, model, token estimate, `reason`, `auth_hint`, and the next `providers diagnose` command, then exits non-zero for automation.

For Kimi auth unblock work, `providers unblock kimi` prints the current source-backed status and the exact verification sequence. `providers inspect-key kimi [--json] [--from-file <new-key-file>]` checks the current or candidate credential offline, never writes the key, never opens the network, prints only a safe fingerprint/shape classification, and uses matching Kimi auth evidence to flag known Kimi Code CLI OAuth material. `--json` returns the same redacted source, fingerprint, classification, policy, status, and next commands for automation. `providers preflight-key kimi --from-file <new-key-file>` then tests a candidate key through the same OpenAI-compatible provider path without writing it to `.kimi` or printing the secret. When the configured endpoint is one of the official Moonshot endpoints, preflight tests both global and China endpoints and prints the exact `MOONSHOT_BASE_URL=... providers rc-unblock` command if only one region passes. `providers rc-unblock kimi --from-file <new-key-file>` now also runs that no-write preflight before installation; if only one official region passes, it reuses that endpoint for the provider test and live Kimi provider dogfood sequence. The two-step path still works: install a key with `providers rotate-key kimi`, then run `providers rc-unblock kimi` from the AgentHub repository. If the first provider test still fails, `providers rc-unblock kimi` runs the Kimi auth check anyway as diagnostics so the redacted two-endpoint auth report is refreshed before returning `blocked`. `scripts/kimi-rc-unblock.sh` remains as a compatible script path and now carries `passed_endpoint` from `kimi-auth-report.json` into provider test retry and provider dogfood.

Kimi Code CLI credentials are not Moonshot API keys. If a source file looks like Kimi CLI OAuth JSON with `access_token` or `refresh_token`, `providers inspect-key kimi`, `providers preflight-key kimi`, `providers rotate-key kimi`, and `scripts/kimi-key-rotate.sh` reject it before any write or provider test and keep token material out of the output.

## Readiness

```bash
agenthub readiness completion --json --check
agenthub readiness next --json --check
agenthub readiness audit --json --check
agenthub readiness blockers --json --check
agenthub readiness checklist --json --check
agenthub readiness evidence --json --check
```

`readiness completion` is the aggregate completion bundle. It combines the final readiness decision with the current action plan, prompt-to-artifact checklist, focused evidence status, raw provider statuses, source files, blocker scope, blocked checks, and verification commands so release automation can answer whether the 1.0 bridge is complete without manually joining several reports.

`readiness next` is the prioritized action-plan view. It uses the same source-backed audit data but collapses it into the current phase, focus, stop reason, next milestone, immediate commands, verification commands, and deferred post-1.0 ecosystem tracks. JSON output is intended for automation that needs one current next step without stitching together audit, checklist, evidence, and ecosystem reports.

`readiness audit` is the full API-native 1.0 gate. JSON output includes source paths, RC evidence metrics, every check row, top-level `blocked_checks`, and per-check `next_commands` for incomplete rows. Text output renders matching `blocked_checks` and `check_next` lines. `readiness blockers` is the short view for humans and automation; it reuses the same recovery commands and emits the same top-level `blocked_checks` summary as the full audit.

`readiness checklist` is the prompt-to-artifact view of the same gate. It groups the API-native 1.0 objective into requirements such as roadmap files, DeepSeek/Kimi API evidence, Chat/Ops/Project mode evidence, memory/observability checks, the RC dogfood gate, and post-1.0 sequencing. Each requirement lists concrete files, commands, mapped readiness checks, blocker kinds, and recovery commands without printing secrets.

`readiness evidence` is the focused RC evidence view. It reports dogfood history thresholds, real session/Ops/project-edit/cost counters, provider dogfood rows, required RC checks, Kimi auth evidence, open blocker state, and the final dogfood gate in one machine-readable report without reading raw JSONL files.

The compatible script path, `scripts/api-native-completion-audit.sh --json --check`, now carries the same `blocker_scope`, `blocker_kinds`, per-check `blocker_kind`, per-check `next_commands`, and top-level `blocked_checks` metadata; text output also prints top-level `blocker_scope`/`blocker_kinds`/`blocked_checks` rows so release automation can distinguish external credential blockers from local implementation gaps without parsing text.

## Ecosystem

```bash
agenthub ecosystem status
agenthub ecosystem status --json
```

`ecosystem status` is a disabled-by-default planning surface for post-1.0 work. It does not connect to MCP/A2A endpoints or enable external protocol runtime. The JSON output lists the planned surfaces from the post-1.0 roadmap: MCP, A2A, Subagents v2, async background agents, Ollama/local LLM, multimodal context, team collaboration, and enterprise/marketplace. Each row includes priority, scope, transports, policy gate, dependencies, acceptance signal, and next implementation files.

## Config

```bash
agenthub config show
agenthub config set default_provider deepseek
```

Configuration is stored in `.agent/config.yaml` as simple key/value settings. `default_provider` falls back to `deepseek` when no config file exists.

`config set` accepts only product-supported keys: `default_provider`, `provider.<id>.template`, `provider.role.<role>`, and `provider.fallback.<role>`. Unknown keys are rejected so typos do not silently change runtime behavior.

## Open

```bash
agenthub open dashboard
agenthub open report tx-20260515123000-abcd1234
```

`open dashboard` refreshes the static dashboard and opens `.agent/reports/dashboard/index.html` when the host has a desktop opener. `open report` opens a transaction `report.md`. In CI or with `AGENTHUB_OPEN_DRY_RUN=1`, AgentHub prints the path without launching an external process.

## Serve

```bash
agenthub serve
agenthub serve --addr 127.0.0.1:4318 --refresh-ms 1000
```

`serve` runs the browser dashboard as a local auto-refresh UI at `http://127.0.0.1:4317` by default. It regenerates dashboard data on requests and is useful while a transaction is running. The dashboard includes the observability payload from `/api/observability`: context receipts, recent chat/provider events, session recovery entries, tool-loop receipts, and tool log excerpts.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
agenthub memory inbox
agenthub memory inbox add "Prefer reviewed memory facts"
agenthub memory inbox approve mem-inbox-12345678 mem-inbox-87654321
agenthub memory inbox reject mem-inbox-12345678,mem-inbox-87654321
```

`inspect` prints raw committed and failed-attempt counts. `summary` is the user-facing view of stack, active decisions, and known failures. `audit` checks stale, conflicting, low-confidence, and unverified records. In Chat/Ops Mode these commands use `$AGENTHUB_HOME/memory` or the platform AgentHub data directory and do not create `.agent`; initialized projects continue to refresh `.agent/memory/audit.json`.

`inbox` is the review-gated memory queue. `add` records a candidate without injecting it into active memory. `agenthub memory inbox` shows a grouped/ranked review view: duplicate/conflict groups, confidence bands, per-candidate confidence, source, summary, and promotion diff preview. `approve` promotes candidates into committed memory, `reject` keeps the audit trail without promotion; both commands accept multiple ids and validate the full batch before promotion so a bad id does not partially apply the batch. Inside the shell, `/memory inbox`, `/memory inbox approve <id...>`, and `/memory inbox reject <id...>` use the same store.

Completed Chat/Ops turns and successful Project transactions may add automatic candidates to this inbox. Each candidate carries source, scope, confidence, evidence excerpts, and diff metadata, and remains inactive until explicit approval.

API chat turns write a compaction receipt at `memory/compacted/context_receipt.json` in the active memory scope. It records selected committed memory, expired records, conflict suppression, budget drops, prompt token estimate, and confirms that pending inbox memory was not injected.

## Ops

```bash
agenthub ops hosts
agenthub ops hosts add prod.example.com --alias prod --trust trusted --note "primary app host"
agenthub ops hosts trust prod.example.com untrusted
agenthub ops runbooks
agenthub ops runbooks add "Check nginx before restart" --host prod.example.com --command "systemctl status nginx"
agenthub ops receipts --host prod.example.com --limit 10
```

`ops hosts` lists host profiles with stable ids, alias/note metadata, trust level, last-seen timestamp, and command count. `ops runbooks` lists reusable runbook cards backed by committed `ops/runbook_step` memory; `add` writes a reviewed memory fact immediately for explicit user-authored runbooks. `ops receipts` lists recent explicit Ops shell commands with target, trust, risk, approval requirement, success, command, and log pointers. Inside the shell, `/ops hosts`, `/ops runbooks`, and `/ops receipts` use the same store.

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` reports each local standard-library skill with analytics-backed runs, success rate, rollback rate, average duration, average cost, and known failure count.
