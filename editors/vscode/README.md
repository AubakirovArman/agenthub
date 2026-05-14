# AgentHub VS Code Extension

This is the Phase 12 v0 IDE surface for AgentHub.

It is intentionally zero-build JavaScript:

- transaction tree over `.agent/tx`;
- memory tree over `.agent/memory`;
- latest report opener;
- DAG webview for `dag.json`;
- prompt-to-AgentSpec preview command backed by `agenthub ask`;
- AgentSpec JSON schema validation.

## Local Development

Open this folder in VS Code and run the extension host from `editors/vscode`.

The extension first tries `agenthub ask`. If the CLI is not installed globally,
it falls back to `cargo run --quiet -- ask` from the workspace root.

