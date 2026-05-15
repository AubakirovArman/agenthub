# Product CLI

语言: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 增加了面向产品使用的命令，用于检查本地安装、provider 状态和简单配置。

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

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
```

支持的 providers：

- `command`: 内置 deterministic command runner。
- `codex`: 外部 Codex CLI wrapper。
- `gemini`: 外部 Gemini CLI wrapper。
- `kimi`: 外部 Kimi CLI wrapper。
- `openai-http`: 本地 OpenAI-compatible HTTP endpoint。

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

`providers test command` 验证内置 runner。CLI providers 会验证 binary discovery、可用时的 version output、以及 template readiness；live authentication 仍由 provider CLI 管理。`providers test openai-http` 会执行真实的 OpenAI-compatible HTTP completion request。

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

配置以简单 key/value settings 形式保存在 `.agent/config.yaml`。没有 config file 时，`default_provider` 默认为 `command`。
