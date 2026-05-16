# V4.13 Local Live Dashboard

## Status

Done for the local preview server foundation.

## Completed

- Added `agenthub serve` with default address `127.0.0.1:4317`.
- Added `/serve` in the chat shell as the interactive shortcut.
- Reused the existing static dashboard payload instead of forking a second UI.
- Regenerated dashboard data on HTTP requests so the browser view can follow transaction history, timeline, metrics, memory, skills, policies, and reports.
- Added dashboard JavaScript live refresh through `window.AGENTHUB_LIVE` and `window.AGENTHUB_REFRESH_MS`.
- Added `--addr`, `--output`, `--refresh-ms`, and `--once` options.
- Extended CLI smoke coverage with a local `/health` request against `agenthub serve --once`.
- Updated dashboard/product/shell docs in English, Russian, Chinese, and Kazakh.

## Evidence

- `cargo test local_server::tests`
- `scripts/smoke-test.sh`

## Remaining V5 Depth

- Add dedicated browser panes for diff/log/report viewing instead of relying only on report links and transaction tables.
- Add approval inbox controls once transaction approval state becomes interactive over HTTP.
- Add optional browser opener for `agenthub serve`.
