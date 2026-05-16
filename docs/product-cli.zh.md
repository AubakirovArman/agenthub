# Product CLI

语言: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 增加了面向产品使用的命令，用于检查本地安装、provider 状态、简单配置和 chat-first local work。

## Chat-first shell

```bash
agenthub
```

不带 subcommand 运行 `agenthub` 是推荐的 daily entry。它可以准备 Git 和 `.agent`，恢复 latest chat，显示 provider readiness，然后让你直接输入普通任务。Shell 会创建 draft plan，询问 inline approval，运行 transaction，然后提示 `/diff`、`/logs`、`/report`、`/explain` 和 `/undo`。

使用 `/` 输入 commands，`@path` 添加 context，`!command` 运行 policy-checked shell command，`# note` 写入 project memory。

## Doctor

```bash
agenthub doctor
```

`doctor` 是安装后的第一个 readiness screen。它检查 AgentHub version、binary path、dev/release channel、OS/architecture、`sh` shell、Git version、Git repository status、`.agent` initialization、policy files、default provider readiness，以及 supported provider binaries/endpoints。缺少 optional Codex/Gemini/Kimi CLI 是 warning；缺少 Git 或 `sh` 是 blocking error。

## Version

```bash
agenthub version
```

输出已安装的 AgentHub 版本。

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run examples/command-task.yaml
```

如果没有提供 `--output`，`plan` 会把 draft AgentSpec 写入 `.agent/drafts/`。`run` 接受已有 AgentSpec path，也接受 natural request。Natural request 会先转换成 draft spec，然后通过正常 transaction engine 执行。

第一行输出保留脚本友好的 `tx-id STATUS (report)` 格式。后续行显示 task、provider、topology、verifier、memory promotion、changed files 数量、report、`tx explain`、`tx watch` 和 dashboard path。

```bash
agenthub tx explain tx-20260515123000-abcd1234
agenthub tx diff tx-20260515123000-abcd1234
agenthub tx logs tx-20260515123000-abcd1234 --tail 80
```

`tx explain` 会概括 transaction 为什么失败或成功、发生了什么、下一步做什么，以及应该查看哪些 artifacts。
`tx diff` 在可用时显示 committed patch，对 uncommitted transactions fallback 到 diff-guard summaries。
`tx logs` 打印 bounded command logs，可按 stage 和 tail length 过滤。

面向单个 transaction 的 commands 可以接收显式 id，也可以接收 `latest`/`last`。这适用于 `tx report`、`tx effects`、`tx explain`、`tx diff`、`tx logs`、`tx watch`、`tx cancel`、`tx resolve`、`tx resume` 和 `tx retry`。

## Undo

```bash
agenthub undo last
agenthub undo tx-20260515123000-abcd1234
```

`undo` 会为 committed AgentHub transaction 创建普通 Git revert。Working tree 有 unrelated uncommitted changes 时它会拒绝运行，并写入 `.agent/tx/<tx-id>/undo.json`。

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers add openai-http --name local-vllm --url http://127.0.0.1:8000 --model qwen3
agenthub providers test codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub providers fallback reviewer gemini kimi openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=https://api.example.com agenthub providers diagnose openai-http
```

支持的 providers：

- `command`: 内置 deterministic command runner。
- `codex`: 外部 Codex CLI wrapper。
- `gemini`: 外部 Gemini CLI wrapper。
- `kimi`: 外部 Kimi CLI wrapper。
- `openai-http`: OpenAI-compatible HTTP 或 HTTPS endpoint。

`setup` 只会在 provider 可用时配置它。成功后会写入 `default_provider`，为 CLI providers 保存 command template，打印 binary 或 endpoint，显示 dry-run mode，并给出下一条 `agenthub ask` command。

示例：

```text
configured	command
default_provider	command
runner	built-in
version	agenthub 0.1.0
dry_run	built-in deterministic runner ready
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers diagnose <id>` 输出 binary 或 endpoint location、可用时的 version、rendered command template、auth hint、status hint、install hint 和 provider-specific details。对 CLI providers，它还会检查已知 credential markers，但不会打印 secret values：Codex 检查 `OPENAI_API_KEY`、`$CODEX_HOME/auth.json` 和 `$HOME/.codex/auth.json`；Gemini 检查 `GEMINI_API_KEY`、`GOOGLE_API_KEY` 和 `$HOME/.gemini`；Kimi 检查 `KIMI_API_KEY`、`MOONSHOT_API_KEY`、`$HOME/.kimi` 和 `$HOME/.config/kimi`。如果没有找到 markers，会显示 `cli_managed_unknown`，因为 provider CLI 仍可能通过其他机制登录。`openai-http` diagnose 会显示 scheme、model、API-key presence，并提示用 `providers test` 做 live request。

`providers set <role> <provider>` 会把 `provider.role.<role>` 保存到 `.agent/config.yaml`。`providers fallback <role> ...` 会把逗号分隔的 fallback chain 保存到 `provider.fallback.<role>`。Valid roles: planner、executor、reviewer、repair、generator、critic、researcher、aggregator、manager、worker。

Named provider profiles 会把 reusable OpenAI-compatible endpoints 保存到 `.agent/config.yaml`：

```bash
agenthub providers add openai-http --name ollama --url http://127.0.0.1:11434 --model qwen3
agenthub providers setup ollama
agenthub providers test ollama
agenthub providers set reviewer ollama
```

Profiles 适合 `local-vllm`、`ollama`、`lm-studio`、`openrouter` 和 company proxy endpoints。可选的 `--api-key-env NAME` 用来告诉 AgentHub bearer token 存在哪个 environment variable。

`providers test command` 验证内置 runner。CLI providers 会验证 binary discovery、可用时的 version output、以及 template readiness；live authentication 仍由 provider CLI 管理。`providers test openai-http` 会执行真实的 OpenAI-compatible HTTP/HTTPS completion request，然后 best-effort 检查 optional `/v1/models`；如果 models endpoint 缺失，会显示 `models unavailable`，不会让 provider test 失败。

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

配置以简单 key/value settings 形式保存在 `.agent/config.yaml`。没有 config file 时，`default_provider` 默认为 `command`。

`config set` 只接受产品支持的 keys：`default_provider`、`provider.<id>.template`、`provider.role.<role>` 和 `provider.fallback.<role>`。未知 key 会被拒绝，避免拼写错误静默改变 runtime behavior。

## Open

```bash
agenthub open dashboard
agenthub open report tx-20260515123000-abcd1234
```

`open dashboard` 会刷新 static dashboard，并在 host 有 desktop opener 时打开 `.agent/reports/dashboard/index.html`。`open report` 会打开指定 transaction 的 `report.md`。在 CI 或设置 `AGENTHUB_OPEN_DRY_RUN=1` 时，AgentHub 只打印 path，不启动 external process。

## Serve

```bash
agenthub serve
agenthub serve --addr 127.0.0.1:4318 --refresh-ms 1000
```

`serve` 会把 browser dashboard 作为 local auto-refresh UI 运行，默认地址是 `http://127.0.0.1:4317`。它会在请求时重新生成 dashboard data，适合 transaction 运行期间保持打开。

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`inspect` 输出 committed 和 failed attempts 的 raw counts。`summary` 是面向用户的 stack、active decisions 和 known failures 视图。`audit` 检查 stale、conflicting、low-confidence 和 unverified records，并刷新 `.agent/memory/audit.json`。

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` 显示每个本地 standard-library skill，并包含 analytics-backed runs、success rate、rollback rate、average duration、average cost 和 known failure count。
