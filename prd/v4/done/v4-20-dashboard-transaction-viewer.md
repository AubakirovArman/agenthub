# V4.20 Dashboard Transaction Viewer

Done for the Dashboard 1.0 inspection loop.

## Scope

- Added `transaction_details` to the dashboard payload.
- Each recent transaction includes bounded report, diff, and log excerpts.
- Added a browser Transaction Viewer section with report, diff, and logs panes.
- Kept static `agenthub dashboard` and live `agenthub serve` on the same payload/assets.
- Kept dashboard JavaScript modules at or below the 200-line limit by adding `dashboard_viewer.js`.
- Updated web dashboard docs and wiki seed in English, Russian, Chinese, and Kazakh.

## Checks

- `cargo fmt -- --check`
- `cargo test web_dashboard::tests --quiet`
- `cargo test local_server::tests --quiet`
- `scripts/check-module-size.sh 200`
