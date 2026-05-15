# PRD Tracker

Source document: [`../prd.md`](../prd.md)

This folder is the working split of the PRD roadmap. The original PRD remains the master text; these files make implementation tracking simpler.

## Rules

1. Work phases in numeric order.
2. Keep unfinished or partially verified phases in `todo/`.
3. Move a phase file to `done/` only after deliverables, acceptance, tests, and 4-language docs are complete.
4. Add the closing commit hash to the phase file when moving it to `done/`.
5. After each phase, run:

```bash
cargo fmt -- --check
cargo test
cargo clippy -- -D warnings
scripts/check-module-size.sh 200
git diff --check
```

## Current Queue

- All tracked PRD phases are in `done/`.
- Long-term open PRD tasks continue in [`todo/`](todo/). Current task: `todo/open-14-plugin-signature-verification.md`.

## Status Index

See [`status.md`](status.md).

## Source Split And Audit

- [`source/`](source/) is a top-level split of the master [`../prd.md`](../prd.md).
- [`audit/`](audit/) records what is done, partial, and open across the full PRD, not only the phase roadmap.
