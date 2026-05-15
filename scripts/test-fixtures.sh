#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-fixtures.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

init_project() {
  local project="$1"
  git -C "$project" init -q
  git -C "$project" config user.email "agenthub@example.invalid"
  git -C "$project" config user.name "AgentHub Fixture"
  git -C "$project" add .
  git -C "$project" commit -q -m "Initial fixture"
  "$AGENTHUB_BIN" --project "$project" init >/dev/null
  git -C "$project" add .agent
  git -C "$project" commit -q -m "Initialize AgentHub"
}

run_fixture() {
  local name="$1"
  local source="$ROOT/fixtures/$name"
  local project="$TMP/$name"
  mkdir -p "$project"
  cp -R "$source/." "$project/"
  init_project "$project"
  local output
  output="$("$AGENTHUB_BIN" --project "$project" run "$project/agenthub-task.yaml")"
  printf '%s\n' "$output"
  if [[ "$output" != *" COMMITTED "* && "$output" != *" COMMITTED ("* ]]; then
    printf 'fixture %s did not commit successfully\n' "$name" >&2
    exit 1
  fi
}

run_reference_web() {
  local project="$TMP/reference-web-app"
  mkdir -p "$project"
  cp -R "$ROOT/examples/reference-web-app/." "$project/"
  init_project "$project"
  local output
  output="$("$AGENTHUB_BIN" --project "$project" run "$ROOT/examples/reference-web-add-courses.yaml")"
  printf '%s\n' "$output"
  test -f "$project/src/app/courses/page.html"
}

run_fixture rust-basic
run_fixture python-data
run_fixture terraform-basic
run_fixture content-basic
run_reference_web

printf 'agenthub fixture smoke tests passed\n'
