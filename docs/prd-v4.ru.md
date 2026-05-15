# PRD v4

Языки: [English](prd-v4.en.md), [Русский](prd-v4.ru.md), [中文](prd-v4.zh.md), [Қазақша](prd-v4.kk.md)

PRD v4 готовит AgentHub к первому tagged local developer preview: `v0.2.0-local-preview`.

## Scope

- Поднять package version до `0.2.0-local-preview`.
- Описать known limitations на четырёх языках.
- Добавить `scripts/dogfood.sh` для повторяемых local product checks.
- Добавить `scripts/release-readiness.sh` для release validation, packaging, local install, `version` и `doctor`.
- Публиковать GitHub Release assets только после зелёного CI на Linux, macOS и Windows.

## Не входит в scope

PRD v4 не выбирает product license, не добавляет hosted SaaS и не утверждает наличие полноценного security sandbox. Это остаётся отдельным product decision или later hardening tracks.
