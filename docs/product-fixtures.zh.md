# Product Fixtures

语言: [English](product-fixtures.en.md), [Русский](product-fixtures.ru.md), [中文](product-fixtures.zh.md), [Қазақша](product-fixtures.kk.md)

PRD v3 product quality fixtures 会在临时 Git projects 上验证 installable CLI paths，而不只依赖 unit tests。

## Fixture Projects

- `fixtures/rust-basic`: 最小 Rust crate，运行 `cargo check`。
- `fixtures/python-data`: data workspace quality artifact。
- `fixtures/terraform-basic`: infra plan artifact，不执行 `terraform apply`。
- `fixtures/content-basic`: content workspace article artifact。
- `examples/reference-web-app`: dependency-free reference web app，用于 `/courses`。

## Smoke Scripts

```bash
scripts/test-fixtures.sh
scripts/test-transaction-rollback.sh
scripts/test-smart-sync.sh
scripts/test-provider-dry-run.sh
scripts/test-dashboard.sh
```

`scripts/test-fixtures.sh` 可安全用于 CI，会运行 Rust、data、infra、content 和 reference web fixtures。其他 scripts 分别检查 rollback、smart sync rebase、adapter dry-run artifacts 和 static dashboard generation。

复用已构建 binary：

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/test-fixtures.sh
```
