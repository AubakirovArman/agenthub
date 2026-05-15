# PRD Audit

Языки: [English](prd-audit.en.md), [Русский](prd-audit.ru.md), [中文](prd-audit.zh.md), [Қазақша](prd-audit.kk.md)

Master PRD находится в [`../prd.md`](../prd.md). Он разложен по top-level sections в [`../prd/source`](../prd/source), а audit выполнения находится в [`../prd/audit`](../prd/audit).

## Результат

Все numbered roadmap phases 1-14 сделаны. Но полный PRD шире, чем phase roadmap: это long-term product vision, поэтому часть не-roadmap пунктов остаётся partial или open.

Использование:

```bash
sed -n '1,160p' prd/audit/status.md
ls prd/source
ls prd/audit/open
```

Главные open areas: formal WAL и reference web fixture.
