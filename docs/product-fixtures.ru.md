# Product Fixtures

Языки: [English](product-fixtures.en.md), [Русский](product-fixtures.ru.md), [中文](product-fixtures.zh.md), [Қазақша](product-fixtures.kk.md)

PRD v3 product quality fixtures проверяют installable CLI paths на временных Git projects, а не только через unit tests.

## Fixture projects

- `fixtures/rust-basic`: минимальный Rust crate, запускает `cargo check`.
- `fixtures/python-data`: data workspace quality artifact.
- `fixtures/terraform-basic`: infra plan artifact без `terraform apply`.
- `fixtures/content-basic`: content workspace article artifact.
- `examples/reference-web-app`: dependency-free reference web app для `/courses`.

## Smoke scripts

```bash
scripts/test-fixtures.sh
scripts/test-transaction-rollback.sh
scripts/test-smart-sync.sh
scripts/test-provider-dry-run.sh
scripts/test-dashboard.sh
```

`scripts/test-fixtures.sh` безопасен для CI и запускает Rust, data, infra, content и reference web fixtures. Остальные scripts проверяют отдельные product risks: rollback, smart sync rebase, adapter dry-run artifacts и static dashboard generation.

Использовать уже собранный binary:

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/test-fixtures.sh
```
