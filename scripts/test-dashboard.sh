#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-dashboard.XXXXXX")"
PROJECT="$TMP/project"
trap 'rm -rf "$TMP"' EXIT INT TERM

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.email "agenthub@example.invalid"
git -C "$PROJECT" config user.name "AgentHub Dashboard"
printf '# dashboard fixture\n' > "$PROJECT/README.md"
git -C "$PROJECT" add .
git -C "$PROJECT" commit -q -m "Initial fixture"
"$AGENTHUB_BIN" --project "$PROJECT" init >/dev/null
git -C "$PROJECT" add .agent
git -C "$PROJECT" commit -q -m "Initialize AgentHub"

"$AGENTHUB_BIN" --project "$PROJECT" run "$ROOT/examples/command-task.yaml" --no-commit >/dev/null
"$AGENTHUB_BIN" --project "$PROJECT" dashboard --output "$TMP/dashboard" >/dev/null
test -f "$TMP/dashboard/index.html"
printf 'agenthub dashboard smoke test passed\n'
