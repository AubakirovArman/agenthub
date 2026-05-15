# Product Fixtures

Тілдер: [English](product-fixtures.en.md), [Русский](product-fixtures.ru.md), [中文](product-fixtures.zh.md), [Қазақша](product-fixtures.kk.md)

PRD v3 product quality fixtures installable CLI paths тек unit tests арқылы емес, уақытша Git projects ішінде тексереді.

## Fixture Projects

- `fixtures/rust-basic`: minimal Rust crate, `cargo check` іске қосады.
- `fixtures/python-data`: data workspace quality artifact.
- `fixtures/terraform-basic`: `terraform apply` қолданбайтын infra plan artifact.
- `fixtures/content-basic`: content workspace article artifact.
- `examples/reference-web-app`: `/courses` үшін dependency-free reference web app.

## Smoke Scripts

```bash
scripts/test-fixtures.sh
scripts/test-transaction-rollback.sh
scripts/test-smart-sync.sh
scripts/test-provider-dry-run.sh
scripts/test-dashboard.sh
```

`scripts/test-fixtures.sh` CI үшін safe және Rust, data, infra, content, reference web fixtures іске қосады. Қалған scripts жеке product risks тексереді: rollback, smart sync rebase, adapter dry-run artifacts және static dashboard generation.

Дайын binary қолдану:

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/test-fixtures.sh
```
