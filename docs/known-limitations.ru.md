# Known Limitations

Языки: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.2.0-local-preview` — устанавливаемый local developer preview, а не стабильный публичный или enterprise product.

## Юридический статус

Репозиторий сейчас `UNLICENSED`. Внешнее использование, копирование, изменение или распространение требует явного разрешения правообладателя, пока владелец проекта не выберет open-source или commercial license.

## Границы sandbox

AgentHub даёт transactional isolation, Git worktrees, command policy checks, rollback, process supervision и hardening reports. Локальные sandbox levels не являются полноценной security boundary для untrusted code. Для рискованных команд используйте remote или isolated runners.

## Providers

CLI providers вроде Codex, Gemini и Kimi находятся через локальные binaries и используют authentication на стороне самого provider. AgentHub может проверить binary presence, version output, templates и dry-run readiness, но не может полностью доказать, что каждый provider account залогинен.

`openai-http` рассчитан на local/dev OpenAI-compatible `http://` endpoints. Direct HTTPS SaaS providers, streaming API calls и provider-specific auth flows запланированы позже.

## Team и Enterprise

Hosted/team surfaces сейчас пишут local export payloads для будущего self-hosted или hosted control plane. Running team server, user accounts, browser login и shared approval inbox пока не реализованы.

## Стабильность release

Release preview может устанавливаться, запускать `doctor`, настраивать provider, выполнять safe transaction и открывать dashboard. API, AAL, plugin и report formats ещё могут измениться до stable release.
