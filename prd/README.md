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

- Current: `todo/phase-10-advanced-agent-topologies.md`
- Then: `todo/phase-11-additional-workspaces.md`
- Then: `todo/phase-12-ide-and-visual-layer.md`
- Then: `todo/phase-13-marketplace-plugin-ecosystem.md`
- Then: `todo/phase-14-enterprise-layer.md`

## Status Index

See [`status.md`](status.md).
