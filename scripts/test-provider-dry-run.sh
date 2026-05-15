#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-provider.XXXXXX")"
PROJECT="$TMP/project"
trap 'rm -rf "$TMP"' EXIT INT TERM

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.email "agenthub@example.invalid"
git -C "$PROJECT" config user.name "AgentHub Provider"
printf '# provider fixture\n' > "$PROJECT/README.md"
git -C "$PROJECT" add .
git -C "$PROJECT" commit -q -m "Initial fixture"
"$AGENTHUB_BIN" --project "$PROJECT" init >/dev/null
git -C "$PROJECT" add .agent
git -C "$PROJECT" commit -q -m "Initialize AgentHub"

output="$("$AGENTHUB_BIN" --project "$PROJECT" run "$ROOT/examples/adapter-dry-run-task.yaml")"
printf '%s\n' "$output"
tx_id="$(printf '%s\n' "$output" | awk '{print $1}')"
test -f "$PROJECT/.agent/tx/$tx_id/adapter_invocation_executor.json"
test -f "$PROJECT/.agent/tx/$tx_id/agent_transcript.jsonl"
printf 'agenthub provider dry-run smoke test passed\n'
