#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-smart-sync.XXXXXX")"
PROJECT="$TMP/project"
SPEC="$TMP/smart-sync-task.yaml"
trap 'rm -rf "$TMP"' EXIT INT TERM

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.email "agenthub@example.invalid"
git -C "$PROJECT" config user.name "AgentHub Smart Sync"
printf '# smart sync fixture\n' > "$PROJECT/README.md"
git -C "$PROJECT" add .
git -C "$PROJECT" commit -q -m "Initial fixture"
"$AGENTHUB_BIN" --project "$PROJECT" init >/dev/null
git -C "$PROJECT" add .agent
git -C "$PROJECT" commit -q -m "Initialize AgentHub"

cat > "$SPEC" <<YAML
task:
  id: smart_sync_independent_fixture
  type: code.command
  title: Rebase independent main change

workspace:
  type: code.git
  isolation: git_worktree

execution:
  commands:
    - mkdir -p generated
    - printf 'tx\n' > generated/tx.txt
    - printf 'main\n' > "$PROJECT/independent.txt"
    - git -C "$PROJECT" add independent.txt
    - git -C "$PROJECT" commit -q -m 'Independent main change'

scope:
  allow:
    - generated/**
  deny:
    - .agent/**

verify:
  commands:
    - test -f generated/tx.txt

transaction:
  commit_on_success: true
  memory_promotion: on_success
YAML

output="$("$AGENTHUB_BIN" --project "$PROJECT" run "$SPEC")"
printf '%s\n' "$output"
if [[ "$output" != *" COMMITTED "* && "$output" != *" COMMITTED ("* ]]; then
  printf 'expected smart sync fixture to commit\n' >&2
  exit 1
fi
test -f "$PROJECT/generated/tx.txt"
test -f "$PROJECT/independent.txt"
printf 'agenthub smart sync smoke test passed\n'
