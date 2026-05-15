# Product Fixtures

Languages: [English](product-fixtures.en.md), [Русский](product-fixtures.ru.md), [中文](product-fixtures.zh.md), [Қазақша](product-fixtures.kk.md)

PRD v3 product quality fixtures prove the installable CLI paths against temporary Git projects, not only unit tests.

## Fixture Projects

- `fixtures/rust-basic`: minimal Rust crate, runs `cargo check`.
- `fixtures/python-data`: data workspace quality artifact.
- `fixtures/terraform-basic`: infra plan artifact without `terraform apply`.
- `fixtures/content-basic`: content workspace article artifact.
- `examples/reference-web-app`: dependency-free reference web app for `/courses`.

## Smoke Scripts

```bash
scripts/test-fixtures.sh
scripts/test-transaction-rollback.sh
scripts/test-smart-sync.sh
scripts/test-provider-dry-run.sh
scripts/test-dashboard.sh
```

`scripts/test-fixtures.sh` is safe for CI and runs the Rust, data, infra, content, and reference web fixtures. The other scripts target specific product risks: rollback, smart sync rebase, adapter dry-run artifacts, and static dashboard generation.

To reuse an existing binary:

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/test-fixtures.sh
```
