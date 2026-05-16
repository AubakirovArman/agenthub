# V4.08 Dogfood History

## Status

Done.

## Completed

- Added `scripts/archive-dogfood.sh` to persist dogfood evidence into `target/dogfood/history/`.
- `scripts/dogfood.sh` now archives each suite report by default after writing `target/dogfood/dogfood-report.json`.
- `scripts/provider-dogfood.sh` now archives direct live provider runs by default after a successful provider transaction.
- The archive writes `index.jsonl`, `latest.json`, and `runs/<run-id>/` directories.
- Provider archives keep the provider report and copied provider artifacts when available.
- Dogfooding docs were updated in English, Russian, Chinese, and Kazakh.

## 1.0 Relevance

This turns one-off dogfood reports into a multi-run evidence trail. It makes 1.0 readiness review less dependent on terminal logs and gives maintainers a stable local history before tagging a final release.
