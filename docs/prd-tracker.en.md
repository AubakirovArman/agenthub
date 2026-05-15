# AgentHub PRD Tracker

The original PRD stays in `prd.md`. The working tracker lives in `prd/`.

Use it like this:

```bash
ls prd/todo
ls prd/done
sed -n '1,120p' prd/status.md
```

Rules:

- Work phases in numeric order.
- Keep partial phases in `prd/todo/`.
- Move a phase to `prd/done/` only after code, tests, acceptance, and 4-language docs are complete.
- Add the closing commit hash to the phase file and `prd/status.md`.

Current next phase: none; all tracked PRD phases are done.

The complete PRD split and completion audit are tracked separately:

- `prd/source/`: top-level split of `prd.md`.
- `prd/audit/`: done, partial, and open PRD areas.
- `prd/todo/open-*.md`: long-term PRD tasks; current task is `open-07-tournament-topology.md`.
