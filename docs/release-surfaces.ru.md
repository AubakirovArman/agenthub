# Release Surfaces

Языки: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

Кроме README в репозитории у AgentHub есть две публичные поверхности документации.

## GitHub Pages

Static product site лежит в `site/` и публикуется workflow `.github/workflows/pages.yml`.

Он нужен для:

- короткого product positioning;
- install и quick-start ссылок;
- ссылок на docs, releases и wiki;
- curated docs hub, который ведёт к canonical Markdown docs;
- страницы 1.0 readiness с командами release gate;
- публичной landing page, которую проще просканировать, чем полный README.

Не копируй весь набор документации в Pages вручную. Pages должен оставаться маленьким порталом; repository Markdown и Wiki остаются canonical editable documentation.

Workflow публикует директорию `site/` через GitHub Pages Actions. Если Pages ещё не включён, включи source GitHub Actions в настройках репозитория.

## Project Wiki

Wiki seed pages лежат в `docs/wiki/`.

Публикация:

```bash
scripts/publish-wiki.sh
```

Скрипт копирует Markdown pages в отдельный репозиторий `agenthub.wiki.git` и делает push. Нужна обычная GitHub git authentication. Для token-based git ставь `AGENTHUB_WIKI_USE_GH_TOKEN=1` с git-compatible `GH_TOKEN`.

First-time note: GitHub может не создать `agenthub.wiki.git`, пока первая wiki page не сохранена в browser. Если publish пишет `Repository not found`, создай первую страницу на `https://github.com/AubakirovArman/agenthub/wiki`, затем перезапусти скрипт.

## Подготовка 1.0

Перед финальным tag используй release preparation script:

```bash
scripts/prepare-1.0-release.sh
```

Ставь `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1`, если скрипт должен падать, пока `scripts/dogfood-readiness.sh --check` не проходит. Ставь `AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1`, когда Kimi auth должен проходить, а не просто показываться как non-enforced preparation blocker.

Для финальной `1.0 RC` репетиции также включай product evidence gate:

```bash
agenthub providers preflight-key kimi --from-file <new-key-file>
agenthub providers rc-unblock kimi --from-file <new-key-file>
scripts/rc-evidence-collect.sh
AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1 AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1 AGENTHUB_PREPARE_REQUIRE_RC_DOGFOOD=1 scripts/prepare-1.0-release.sh
```

`providers preflight-key kimi --from-file <new-key-file>` проверяет candidate key без записи и без вывода secret. На official Moonshot endpoint-ах команда проверяет и global, и China region, а затем печатает точную команду `MOONSHOT_BASE_URL=... providers rc-unblock` для прошедшего endpoint-а. `providers rc-unblock kimi --from-file <new-key-file>` теперь повторяет этот no-write preflight перед установкой replacement key, затем использует прошедший endpoint для Kimi provider test, live Kimi provider dogfood, RC evidence collection и RC gate в правильном порядке. Если provider test падает, команда всё равно запускает Kimi auth diagnostics, чтобы redacted two-endpoint auth report был актуальным перед возвратом `blocked`. Preparation gate запускает `scripts/rc-dogfood-gate.sh --check`: он требует real-session evidence для Chat/Ops/Project usage, provider dogfood для DeepSeek/Kimi, cost receipts, resume/rewind/stats checks, отсутствие Chat/Ops bootstrap side effects и отсутствие open blocker/critical release issues.
