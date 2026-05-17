# Release Surfaces

语言: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

除了 repository README，AgentHub 还有两个公开文档入口。

## GitHub Pages

Static product site 位于 `site/`，由 `.github/workflows/pages.yml` 发布。

用途：

- 简短 product positioning；
- install 和 quick-start links；
- docs、releases 和 wiki links；
- 指向 canonical Markdown docs 的 curated docs hub；
- 包含 release gate commands 的 1.0 readiness page；
- 比完整 README 更容易浏览的 public landing page。

不要把完整 documentation set 手动复制到 Pages。Pages 应保持为小型 portal；repository Markdown 和 Wiki 仍是 canonical editable documentation。

Workflow 使用 GitHub Pages Actions 发布 `site/` 目录。如果 Pages 尚未启用，在 repository settings 中把 source 设置为 GitHub Actions。

## Project Wiki

Wiki seed pages 位于 `docs/wiki/`。

发布：

```bash
scripts/publish-wiki.sh
```

脚本会把 Markdown pages 复制到单独的 `agenthub.wiki.git` repository 并 push。需要普通 GitHub git authentication。Token-based git 可设置 `AGENTHUB_WIKI_USE_GH_TOKEN=1` 和 git-compatible `GH_TOKEN`。

First-time note: GitHub 可能要等第一篇 wiki page 在 browser 中保存后才会创建 `agenthub.wiki.git`。如果 publish 显示 `Repository not found`，先在 `https://github.com/AubakirovArman/agenthub/wiki` 创建第一页，然后重新运行脚本。

## 1.0 准备

创建最终 tag 前运行 release preparation script：

```bash
scripts/prepare-1.0-release.sh
```

如果希望 dogfood readiness 未通过时脚本失败，设置 `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1`。Set `AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1` when Kimi auth must pass instead of being reported as a non-enforced preparation blocker.

For a final 1.0 RC rehearsal, also require the product evidence gate:

```bash
agenthub providers preflight-key kimi --from-file <new-key-file>
agenthub providers rc-unblock kimi --from-file <new-key-file>
scripts/rc-evidence-collect.sh
AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1 AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1 AGENTHUB_PREPARE_REQUIRE_RC_DOGFOOD=1 scripts/prepare-1.0-release.sh
```

`providers preflight-key kimi --from-file <new-key-file>` tests the candidate key without writing it or printing the secret. On official Moonshot endpoints it checks both global and China regions and prints the exact `MOONSHOT_BASE_URL=... providers rc-unblock` command for the passing endpoint. `providers rc-unblock kimi --from-file <new-key-file>` now repeats that no-write preflight before installing the replacement key, then reuses the passing endpoint for the Kimi provider test, live Kimi provider dogfood, RC evidence collection, and the RC gate in the required order. If the provider test fails, it still runs Kimi auth diagnostics so the redacted two-endpoint auth report is current before the command returns `blocked`. The preparation gate runs `scripts/rc-dogfood-gate.sh --check`, which requires real-session evidence for Chat/Ops/Project usage, provider dogfood for DeepSeek/Kimi, cost receipts, resume/rewind/stats checks, no Chat/Ops bootstrap side effects, and no open blocker/critical release issues.
