#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMPDIR:-$ROOT/target/tmp}"
mkdir -p "$TMP_ROOT"
TMP="$(mktemp -d "$TMP_ROOT/agenthub-shell-ux.XXXXXX")"
PROJECT="$TMP/project"
HOME_DIR="$TMP/home"
CONFIG_DIR="$TMP/config"
OUT="$TMP/shell.out"
ERR="$TMP/shell.err"
TUI_OUT="$TMP/tui.out"
TUI_ERR="$TMP/tui.err"

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

run_agenthub() {
  if [[ -n "${AGENTHUB_BIN:-}" ]]; then
    "$AGENTHUB_BIN" --project "$PROJECT" "$@"
  else
    cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -- --project "$PROJECT" "$@"
  fi
}

require_file_output() {
  local file="$1"
  local err_file="$2"
  local pattern="$3"
  if ! grep -Fq -- "$pattern" "$file"; then
    printf 'expected output to contain: %s\n' "$pattern" >&2
    printf '%s\n' '--- stdout ---' >&2
    sed -n '1,240p' "$file" >&2
    printf '%s\n' '--- stderr ---' >&2
    sed -n '1,120p' "$err_file" >&2
    exit 1
  fi
}

require_output() {
  require_file_output "$OUT" "$ERR" "$1"
}

require_tui_output() {
  require_file_output "$TUI_OUT" "$TUI_ERR" "$1"
}

mkdir -p "$PROJECT" "$HOME_DIR" "$CONFIG_DIR"

{
  printf '/mode chat\n'
  printf '/mode devops\n'
  printf '/mode project\n'
  printf '/sessions\n'
  printf '/cost\n'
  printf '/balance\n'
  printf '/hosts\n'
  printf '/connect shell-smoke-host\n'
  printf '/provider deepseek\n'
  printf '!printf shell-smoke-ok\n'
  printf '/exit\n'
} | AGENTHUB_HOME="$HOME_DIR" \
    XDG_CONFIG_HOME="$CONFIG_DIR" \
    GIT_CEILING_DIRECTORIES="$TMP" \
    DEEPSEEK_API_KEY="shell-smoke-key" \
    run_agenthub shell >"$OUT" 2>"$ERR"

require_output 'Mode: chat  Git: not required  .agent: not required'
require_output $'workspace_mode\tchat'
require_output $'workspace_mode\tops'
require_output $'workspace_mode\tproject\tpending_runtime'
require_output 'Chats'
require_output 'Chat Usage'
require_output $'provider_balance\tnot_available'
require_output 'Ops hosts:'
require_output $'host\tops-host-shell-smoke-host'
require_output $'selected\tdeepseek'
require_output $'default_provider\tdeepseek'
require_output 'tool_permission tool=shell profile=read-only'
require_output 'shell-smoke-ok'

AGENTHUB_HOME="$HOME_DIR" \
  XDG_CONFIG_HOME="$CONFIG_DIR" \
  GIT_CEILING_DIRECTORIES="$TMP" \
  DEEPSEEK_API_KEY="shell-smoke-key" \
  run_agenthub tui >"$TUI_OUT" 2>"$TUI_ERR"

require_tui_output 'AgentHub TUI Dashboard'
require_tui_output '[Status Line]'
require_tui_output 'provider: deepseek ok model:deepseek-chat | git optional | global session'
require_tui_output '[Composer]'
require_tui_output '/providers   inspect DeepSeek/Kimi API setup'
require_tui_output '[Chat Transcript]'
require_tui_output 'tool: shell classified as read-only'
require_tui_output '[Event Rail]'
require_tui_output '[ready] tool permission: read-only risk low approval false'
require_tui_output '[event] ops_command: printf shell-smoke-ok'
require_tui_output '[Live Tool Cards]'
require_tui_output '[ready] tool_permission: shell read-only'
require_tui_output 'risk low approval false action printf shell-smoke-ok'
require_tui_output '[Summary]'
require_tui_output '- total transactions: 0'
require_tui_output '[Providers]'
require_tui_output 'deepseek [ok default]'
require_tui_output '[Next Actions]'
require_tui_output 'agenthub run "describe the change" --no-commit'

test ! -e "$PROJECT/.git"
test ! -e "$PROJECT/.agent"
test -f "$CONFIG_DIR/agenthub/config.yaml"
grep -Fq 'default_provider: deepseek' "$CONFIG_DIR/agenthub/config.yaml"
test -d "$HOME_DIR/sessions"

printf 'shell_aliases\tpassed\n'
printf 'tui_snapshot\tpassed\n'
printf 'no_bootstrap\tpassed\n'
printf 'agenthub shell UX alias smoke passed\n'
