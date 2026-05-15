# Phase 12 — IDE and Visual Layer

Status: Done

Existing evidence: `58a68b8 Add VS Code IDE surface`

Closing evidence: `d3517eb`.

## Deliverables

- VS Code extension: done.
- Transaction panel: done.
- Memory panel: done.
- AgentSpec editor: done.
- Visual DAG viewer: done.
- Approval UI: done.

## Acceptance

- Developer can inspect and manage AgentHub from IDE: done.

## Verification

- Audited existing VS Code extension against all deliverables.
- Added AgentSpecs view for `.agent/specs` drafts and examples.
- Added Approvals view for approval-required specs and blocked transactions.
- Added approve/run/run-without-commit UI command.
- Added VS Code CLI fallback module and pure approval/spec listing tests.
- Added 4-language IDE docs with usage examples.
- Kept JS modules under 200 lines.
