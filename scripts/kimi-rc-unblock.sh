#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AGENTHUB_BIN="${AGENTHUB_BIN:-agenthub}"
KIMI_AUTH_CHECK_CMD="${AGENTHUB_KIMI_AUTH_CHECK_CMD:-$ROOT/scripts/kimi-auth-check.sh}"
PROVIDER_DOGFOOD_CMD="${AGENTHUB_PROVIDER_DOGFOOD_CMD:-$ROOT/scripts/provider-dogfood.sh}"
RC_EVIDENCE_COLLECT_CMD="${AGENTHUB_RC_EVIDENCE_COLLECT_CMD:-$ROOT/scripts/rc-evidence-collect.sh}"
RC_DOGFOOD_GATE_CMD="${AGENTHUB_RC_DOGFOOD_GATE_CMD:-$ROOT/scripts/rc-dogfood-gate.sh}"
SKIP_PROVIDER_DOGFOOD=false
NO_CHECK=false

usage() {
  cat <<'USAGE'
Usage:
  scripts/kimi-rc-unblock.sh [--skip-provider-dogfood] [--no-check]

Runs the Kimi 1.0 RC unblock path after a key has been rotated:
  1. agenthub providers test kimi
  2. scripts/kimi-auth-check.sh
  3. live Kimi provider dogfood
  4. scripts/rc-evidence-collect.sh
  5. scripts/rc-dogfood-gate.sh --check

If the provider test fails, the auth check still runs as diagnostics so the
redacted auth report covers both official Moonshot endpoints before the command
returns blocked.

If you have a replacement key file, verify it without writing first:
  agenthub providers preflight-key kimi --from-file <new-key-file>

Then run the one product-CLI unblock command:
  agenthub providers rc-unblock kimi --from-file <new-key-file>
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-provider-dogfood)
      SKIP_PROVIDER_DOGFOOD=true
      ;;
    --no-check)
      NO_CHECK=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

run_required() {
  local label="$1"
  shift
  printf 'step\t%s\tbegin\n' "$label"
  if "$@"; then
    printf 'step\t%s\tpassed\n' "$label"
  else
    local code=$?
    printf 'step\t%s\tfailed\t%s\n' "$label" "$code"
    return "$code"
  fi
}

printf 'AgentHub Kimi RC unblock\n'

if ! run_required provider_test "$AGENTHUB_BIN" providers test kimi; then
  run_required kimi_auth_check "$KIMI_AUTH_CHECK_CMD" || true
  printf 'status\tblocked\n'
  printf 'reason\tprovider_test_failed\n'
  printf 'next\t1\tagenthub providers preflight-key kimi --from-file <new-key-file>\n'
  printf 'next\t2\tagenthub providers rc-unblock kimi --from-file <new-key-file>\n'
  printf 'next\t3\tagenthub providers rotate-key kimi --from-file <new-key-file>\n'
  printf 'next\t4\tagenthub providers unblock kimi\n'
  exit 1
fi

if ! run_required kimi_auth_check "$KIMI_AUTH_CHECK_CMD"; then
  printf 'status\tblocked\n'
  printf 'reason\tkimi_auth_check_failed\n'
  printf 'next\t1\tagenthub providers preflight-key kimi --from-file <new-key-file>\n'
  printf 'next\t2\tagenthub providers rc-unblock kimi --from-file <new-key-file>\n'
  printf 'next\t3\tagenthub providers rotate-key kimi --from-file <new-key-file>\n'
  printf 'next\t4\tagenthub providers unblock kimi\n'
  exit 1
fi

if [[ "$SKIP_PROVIDER_DOGFOOD" == true ]]; then
  printf 'step\tprovider_dogfood\tskipped\n'
  printf 'warning\tprovider_dogfood_required_for_rc_gate\n'
else
  if ! AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi \
    AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 \
    run_required provider_dogfood "$PROVIDER_DOGFOOD_CMD"; then
    printf 'status\tblocked\n'
    printf 'reason\tprovider_dogfood_failed\n'
    printf 'next\t1\tAGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh\n'
    exit 1
  fi
fi

run_required rc_evidence_collect "$RC_EVIDENCE_COLLECT_CMD"

if [[ "$NO_CHECK" == true ]]; then
  run_required rc_dogfood_gate_summary "$RC_DOGFOOD_GATE_CMD"
  printf 'status\tunchecked\n'
  printf 'next\t1\tscripts/rc-dogfood-gate.sh --check\n'
else
  run_required rc_dogfood_gate "$RC_DOGFOOD_GATE_CMD" --check
  printf 'status\tready\n'
fi
