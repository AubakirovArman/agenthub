#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-rollback.XXXXXX")"
PROJECT="$TMP/project"
SPEC="$TMP/rollback-task.yaml"
trap 'rm -rf "$TMP"' EXIT INT TERM

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.email "agenthub@example.invalid"
git -C "$PROJECT" config user.name "AgentHub Rollback"
printf '# rollback fixture\n' > "$PROJECT/README.md"
git -C "$PROJECT" add .
git -C "$PROJECT" commit -q -m "Initial fixture"
"$AGENTHUB_BIN" --project "$PROJECT" init >/dev/null
git -C "$PROJECT" add .agent
git -C "$PROJECT" commit -q -m "Initialize AgentHub"

cat > "$SPEC" <<'YAML'
task:
  id: rollback_scope_fixture
  type: code.command
  title: Prove rollback on out-of-scope diff

workspace:
  type: code.git
  isolation: git_worktree

execution:
  commands:
    - mkdir -p allowed blocked
    - printf 'ok\n' > allowed/ok.txt
    - printf 'blocked\n' > blocked/out.txt

scope:
  allow:
    - allowed/**
  deny:
    - blocked/**
    - .agent/**

verify:
  commands:
    - true

transaction:
  rollback_on_failure: true
  commit_on_success: true
YAML

output="$("$AGENTHUB_BIN" --project "$PROJECT" run "$SPEC")"
printf '%s\n' "$output"
if [[ "$output" != *" ROLLED_BACK "* && "$output" != *" ROLLED_BACK ("* ]]; then
  printf 'expected rollback transaction to finish as ROLLED_BACK\n' >&2
  exit 1
fi
test ! -e "$PROJECT/blocked/out.txt"
printf 'agenthub rollback smoke test passed\n'
