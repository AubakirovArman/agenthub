# Release Surfaces

语言: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

除了 repository README，AgentHub 还有两个公开文档入口。

## GitHub Pages

Static product site 位于 `site/`，由 `.github/workflows/pages.yml` 发布。

用途：

- 简短 product positioning；
- install 和 quick-start links；
- docs、releases 和 wiki links；
- 比完整 README 更容易浏览的 public landing page。

Workflow 使用 GitHub Pages Actions 发布 `site/` 目录。如果 Pages 尚未启用，在 repository settings 中把 source 设置为 GitHub Actions。

## Project Wiki

Wiki seed pages 位于 `docs/wiki/`。

发布：

```bash
scripts/publish-wiki.sh
```

脚本会把 Markdown pages 复制到单独的 `agenthub.wiki.git` repository 并 push。需要普通 GitHub git authentication 或 `GH_TOKEN`。

## 1.0 准备

创建最终 tag 前运行 release preparation script：

```bash
scripts/prepare-1.0-release.sh
```

如果希望 dogfood readiness 未通过时脚本失败，设置 `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1`。
