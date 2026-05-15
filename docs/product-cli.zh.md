# Product CLI

语言: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 增加了面向产品使用的命令，用于检查本地安装、provider 状态和简单配置。

## Doctor

```bash
agenthub doctor
```

`doctor` 检查 OS/architecture、Git 是否可用、Git repository 状态、`.agent` initialization、policy files 和支持的 provider binaries。缺少 Codex/Gemini/Kimi CLI 会显示 warning，不会作为 blocking error。

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
```

支持的 providers：

- `command`: 内置 deterministic command runner。
- `codex`: 外部 Codex CLI wrapper。
- `gemini`: 外部 Gemini CLI wrapper。
- `kimi`: 外部 Kimi CLI wrapper。

`setup` 只会在 provider 可用时配置它。如果 binary 缺失，AgentHub 会输出可操作的 install/PATH message。

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

配置以简单 key/value settings 形式保存在 `.agent/config.yaml`。没有 config file 时，`default_provider` 默认为 `command`。
