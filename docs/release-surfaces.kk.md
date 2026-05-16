# Release Surfaces

Тілдер: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

Repository README-ден бөлек AgentHub үшін екі public documentation surface бар.

## GitHub Pages

Static product site `site/` ішінде, оны `.github/workflows/pages.yml` жариялайды.

Ол мыналар үшін керек:

- қысқа product positioning;
- install және quick-start links;
- docs, releases және wiki links;
- толық README-ге қарағанда тезірек оқылатын public landing page.

Workflow `site/` директориясын GitHub Pages Actions арқылы publish етеді. Pages әлі қосылмаса, repository settings ішінде source ретінде GitHub Actions таңда.

## Project Wiki

Wiki seed pages `docs/wiki/` ішінде.

Жариялау:

```bash
scripts/publish-wiki.sh
```

Скрипт Markdown pages файлдарын бөлек `agenthub.wiki.git` repository ішіне көшіріп, push жасайды. Қалыпты GitHub git authentication керек. Token-based git үшін `AGENTHUB_WIKI_USE_GH_TOKEN=1` және git-compatible `GH_TOKEN` қой.

First-time note: GitHub бірінші wiki page browser ішінде сақталғанға дейін `agenthub.wiki.git` жасамауы мүмкін. Publish `Repository not found` десе, алдымен `https://github.com/AubakirovArman/agenthub/wiki` ішінде бірінші page жаса, содан кейін script қайта іске қос.

## 1.0 Дайындау

Final tag алдында release preparation script іске қос:

```bash
scripts/prepare-1.0-release.sh
```

Dogfood readiness өтпесе script fail болсын десең, `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1` қой.
