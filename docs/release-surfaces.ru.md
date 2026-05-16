# Release Surfaces

Языки: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

Кроме README в репозитории у AgentHub есть две публичные поверхности документации.

## GitHub Pages

Static product site лежит в `site/` и публикуется workflow `.github/workflows/pages.yml`.

Он нужен для:

- короткого product positioning;
- install и quick-start ссылок;
- ссылок на docs, releases и wiki;
- публичной landing page, которую проще просканировать, чем полный README.

Workflow публикует директорию `site/` через GitHub Pages Actions. Если Pages ещё не включён, включи source GitHub Actions в настройках репозитория.

## Project Wiki

Wiki seed pages лежат в `docs/wiki/`.

Публикация:

```bash
scripts/publish-wiki.sh
```

Скрипт копирует Markdown pages в отдельный репозиторий `agenthub.wiki.git` и делает push. Нужна обычная GitHub git authentication или `GH_TOKEN`.

## Подготовка 1.0

Перед финальным tag используй release preparation script:

```bash
scripts/prepare-1.0-release.sh
```

Ставь `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1`, если скрипт должен падать, пока `scripts/dogfood-readiness.sh --check` не проходит.
