#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="${AGENTHUB_WIKI_REPO:-AubakirovArman/agenthub}"
SOURCE="${AGENTHUB_WIKI_SOURCE:-$ROOT/docs/wiki}"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-wiki.XXXXXX")"
WORK="$TMP/wiki"
REMOTE="https://github.com/${REPO}.wiki.git"
BRANCH="${AGENTHUB_WIKI_BRANCH:-master}"

git_auth_args=()
if [[ -n "${GH_TOKEN:-}" ]]; then
  git_auth_args=(-c "http.https://github.com/.extraheader=AUTHORIZATION: bearer $GH_TOKEN")
fi

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT INT TERM

if [[ ! -d "$SOURCE" ]]; then
  printf 'wiki source directory not found: %s\n' "$SOURCE" >&2
  exit 1
fi

if ! git "${git_auth_args[@]}" clone "$REMOTE" "$WORK" >/dev/null 2>&1; then
  mkdir -p "$WORK"
  git -C "$WORK" init -q
  git -C "$WORK" remote add origin "$REMOTE"
fi

find "$WORK" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
cp "$SOURCE"/*.md "$WORK"/

git -C "$WORK" config user.email "${AGENTHUB_WIKI_GIT_EMAIL:-agenthub@example.invalid}"
git -C "$WORK" config user.name "${AGENTHUB_WIKI_GIT_NAME:-AgentHub Wiki Bot}"
git -C "$WORK" add .

if git -C "$WORK" diff --cached --quiet; then
  printf 'agenthub wiki is already up to date\n'
  exit 0
fi

git -C "$WORK" commit -q -m "${AGENTHUB_WIKI_COMMIT_MESSAGE:-Update AgentHub wiki}"
git -C "$WORK" "${git_auth_args[@]}" push origin "HEAD:$BRANCH"
printf 'agenthub wiki published to %s\n' "$REMOTE"
