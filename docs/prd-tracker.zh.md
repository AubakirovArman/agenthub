# AgentHub PRD Tracker

原始 PRD 保留在 `prd.md`。工作 tracker 位于 `prd/`。

使用方式：

```bash
ls prd/todo
ls prd/done
sed -n '1,120p' prd/status.md
```

规则：

- 按数字顺序完成 phases。
- Partial phases 保留在 `prd/todo/`。
- 只有 code、tests、acceptance 和 4-language docs 完成后，才能移动到 `prd/done/`。
- 在 phase 文件和 `prd/status.md` 中记录 closing commit hash。

当前下一阶段：无；所有 tracked PRD phases 已完成。
