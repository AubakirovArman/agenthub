# AgentHub Wiki

AgentHub — AI coding agents үшін жергілікті transactional runtime. Ол Codex, Gemini, Kimi, command adapters және OpenAI-compatible endpoints құралдарын isolated worktrees, verifier checks, rollback, memory, reports және dashboards арқылы басқарады.

Тілдер: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Quick Start

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub
```

Содан кейін ordinary task жаз:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
agenthub> create a Django web application
```

## Күнделікті workflow

- `agenthub` subcommand жоқ іске қосылса chat-first local shell ашылады.
- Shell ішінде `/cd <folder>` project-ті restart жасамай ауыстырады.
- Бірінші іске қосу fresh project үшін Git, `.agent`, baseline commit және bundled standard skills дайындай алады.
- Interactive `agenthub run` және shell task execution live journal progress көрсетеді; quiet scripts үшін `--no-watch` қолдан.
- Shell ішінде `/providers` provider wizard ретінде қолдан, содан кейін `/status`, `/diff`, `/logs`, `/report`, `/explain` және `/dashboard` қолдан.
- Local auto-refresh dashboard үшін `/serve` немесе `agenthub serve` қолдан.
- Transactions, providers, memory, approvals және next actions бар terminal dashboard үшін `agenthub tui --live` қолдан.
- Dashboard provider status, approval inbox, memory browser, history browser және report, diff, logs үшін transaction viewer panes береді.
- Structured language diagnostics, supported workspace/topology hints және golden AgentIR/DAG checks үшін `agenthub aal check <file.aal>` қолдан.
- Natural language bounded files, Next.js pages және verifier checks бар Django starter scaffold жасай алады.
- Reusable local model endpoints сақтау үшін `agenthub providers add openai-http --name local-vllm --url ...` қолдан.
- Kimi API тікелей тексеру: `KIMI_API_KEY=... agenthub providers test kimi-api`.
- Auto titles бар chat sessions басқару үшін `/chats`, `/search`, `/rename`, `/pin` және `/unpin` қолдан; filter: `/chats status:COMMITTED provider:codex date:today`.
- `/context` current chat, recent messages, memory және selected transaction context preview көрсетеді.
- Approval prompts risk көрсетеді және `diff`, `details`, `edit` қолдайды.
- `@path`, `@tx:<id>` және `@memory:<query>` context қосады, `!command` policy-checked shell command іске қосады, `# note` project memory сақтайды.
- `agenthub run`, `agenthub tx diff latest` және `agenthub tx logs latest` сияқты scriptable commands қолжетімді.
- Release work алдында `scripts/dogfood.sh` және `scripts/dogfood-readiness.sh` іске қос.

## Негізгі сілтемелер

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
