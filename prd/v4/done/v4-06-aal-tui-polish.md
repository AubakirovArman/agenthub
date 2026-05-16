# V4.06 AAL And TUI Polish

## Status

Done.

## Completed

- Added `agenthub aal format` with stdout, `--output`, and `--check` modes.
- AAL semantic diagnostics now preserve line numbers for skills, scope overlaps, verifier profile errors, and runtime smoke warnings.
- CLI diagnostics print source line snippets when a diagnostic has a line number.
- TUI now includes a summary panel with transaction counts by state.
- TUI now includes next-action suggestions for latest, blocked, failed, rolled-back, and empty transaction states.
- Updated AAL and TUI docs in English, Russian, Chinese, and Kazakh.

## 1.0 Relevance

AAL formatting makes review and CI usage predictable. Better diagnostics reduce authoring friction. TUI summary and next actions make local status inspection more useful during daily dogfooding.
