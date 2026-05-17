#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-kimi-rc-unblock.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

cat > "$TMP/agenthub-ok" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$AGENTHUB_KIMI_RC_TEST_LOG"
if [[ "$1 $2 $3" == "providers test kimi" ]]; then
  printf 'ok\tkimi\tcompletion_tokens:1\n'
  exit 0
fi
exit 2
SH
chmod +x "$TMP/agenthub-ok"

cat > "$TMP/agenthub-fail" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$AGENTHUB_KIMI_RC_TEST_LOG"
printf 'failed\tkimi\tauth\n'
exit 1
SH
chmod +x "$TMP/agenthub-fail"

cat > "$TMP/kimi-auth" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'kimi-auth\n' >> "$AGENTHUB_KIMI_RC_TEST_LOG"
printf 'AgentHub Kimi auth check\nstatus\tpassed\n'
SH
chmod +x "$TMP/kimi-auth"

cat > "$TMP/provider-dogfood" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'provider-dogfood provider=%s live=%s\n' "${AGENTHUB_PROVIDER_DOGFOOD_PROVIDER:-}" "${AGENTHUB_PROVIDER_DOGFOOD_LIVE:-}" >> "$AGENTHUB_KIMI_RC_TEST_LOG"
printf 'agenthub provider dogfood passed: tx-demo committed\n'
SH
chmod +x "$TMP/provider-dogfood"

cat > "$TMP/collect" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'collect\n' >> "$AGENTHUB_KIMI_RC_TEST_LOG"
printf 'collected\n'
SH
chmod +x "$TMP/collect"

cat > "$TMP/gate" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'gate %s\n' "$*" >> "$AGENTHUB_KIMI_RC_TEST_LOG"
printf '1.0 RC dogfood gate: ready\n'
SH
chmod +x "$TMP/gate"

log="$TMP/success.log"
AGENTHUB_KIMI_RC_TEST_LOG="$log" \
AGENTHUB_BIN="$TMP/agenthub-ok" \
AGENTHUB_KIMI_AUTH_CHECK_CMD="$TMP/kimi-auth" \
AGENTHUB_PROVIDER_DOGFOOD_CMD="$TMP/provider-dogfood" \
AGENTHUB_RC_EVIDENCE_COLLECT_CMD="$TMP/collect" \
AGENTHUB_RC_DOGFOOD_GATE_CMD="$TMP/gate" \
  "$ROOT/scripts/kimi-rc-unblock.sh" > "$TMP/success.out"

grep -q $'step\tprovider_test\tpassed' "$TMP/success.out"
grep -q $'step\tkimi_auth_check\tpassed' "$TMP/success.out"
grep -q $'step\tprovider_dogfood\tpassed' "$TMP/success.out"
grep -q $'step\trc_evidence_collect\tpassed' "$TMP/success.out"
grep -q $'step\trc_dogfood_gate\tpassed' "$TMP/success.out"
grep -q $'status\tready' "$TMP/success.out"
grep -q 'providers test kimi' "$log"
grep -q 'provider-dogfood provider=kimi live=1' "$log"
grep -q 'gate --check' "$log"

fail_log="$TMP/fail.log"
if AGENTHUB_KIMI_RC_TEST_LOG="$fail_log" \
  AGENTHUB_BIN="$TMP/agenthub-fail" \
  AGENTHUB_KIMI_AUTH_CHECK_CMD="$TMP/kimi-auth" \
  AGENTHUB_PROVIDER_DOGFOOD_CMD="$TMP/provider-dogfood" \
  AGENTHUB_RC_EVIDENCE_COLLECT_CMD="$TMP/collect" \
  AGENTHUB_RC_DOGFOOD_GATE_CMD="$TMP/gate" \
    "$ROOT/scripts/kimi-rc-unblock.sh" > "$TMP/fail.out" 2>&1; then
  printf 'expected Kimi RC unblock to fail when provider test fails\n' >&2
  exit 1
fi
grep -q $'status\tblocked' "$TMP/fail.out"
grep -q $'reason\tprovider_test_failed' "$TMP/fail.out"
grep -q $'step\tkimi_auth_check\tbegin' "$TMP/fail.out"
grep -q $'step\tkimi_auth_check\tpassed' "$TMP/fail.out"
grep -q $'next\t1\tagenthub providers preflight-key kimi --from-file <new-key-file>' "$TMP/fail.out"
grep -q $'next\t2\tagenthub providers rc-unblock kimi --from-file <new-key-file>' "$TMP/fail.out"
grep -q $'next\t4\tagenthub providers unblock kimi' "$TMP/fail.out"
grep -q 'kimi-auth' "$fail_log"
if grep -q 'provider-dogfood' "$fail_log"; then
  printf 'provider dogfood should not run after failed provider test\n' >&2
  exit 1
fi

printf 'agenthub Kimi RC unblock test passed\n'
