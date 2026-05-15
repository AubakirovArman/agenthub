# Known Limitations

Тілдер: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.2.0-local-preview` — installable local developer preview, stable public немесе enterprise product емес.

## Құқықтық статус

Repository қазір `UNLICENSED`. Project owner open-source немесе commercial license таңдағанға дейін external use, copy, modification немесе redistribution үшін copyright holder нақты рұқсаты керек.

## Sandbox шекарасы

AgentHub transactional isolation, Git worktrees, command policy checks, rollback, process supervision және hardening reports береді. Local sandbox levels untrusted code үшін толық security boundary емес. Risky commands үшін remote немесе isolated runners қолданыңыз.

## Providers

Codex, Gemini және Kimi сияқты CLI providers local binaries арқылы табылады және authentication provider CLI жағында басқарылады. AgentHub binary presence, version output, templates және dry-run readiness тексере алады, бірақ әр provider account logged in екенін толық дәлелдей алмайды.

`openai-http` local/dev OpenAI-compatible `http://` endpoints үшін арналған. Direct HTTPS SaaS providers, streaming API calls және provider-specific auth flows кейінгі нұсқаларға жоспарланған.

## Team және Enterprise

Hosted/team surfaces қазір future self-hosted немесе hosted control plane үшін local export payloads жазады. Running team server, user accounts, browser login және shared approval inbox әлі жоқ.

## Release тұрақтылығы

Release preview орнатылады, `doctor` іске қосады, provider конфигурациялайды, safe transaction орындайды және dashboard ашады. API, AAL, plugin және report formats stable release-ке дейін өзгеруі мүмкін.
