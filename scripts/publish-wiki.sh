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
if [[ "${AGENTHUB_WIKI_USE_GH_TOKEN:-0}" == "1" && -n "${GH_TOKEN:-}" ]]; then
  auth="$(printf 'x-access-token:%s' "$GH_TOKEN" | base64 | tr -d '\n')"
  git_auth_args=(-c "http.https://github.com/.extraheader=AUTHORIZATION: basic $auth")
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
if ! git -C "$WORK" "${git_auth_args[@]}" push origin "HEAD:$BRANCH"; then
  cat >&2 <<TEXT
agenthub wiki publish failed.

If GitHub says "Repository not found", open:
  https://github.com/${REPO}/wiki

Create and save the first wiki page once, then rerun:
  scripts/publish-wiki.sh

If authentication failed, run gh auth setup-git or set a git-compatible token with:
  AGENTHUB_WIKI_USE_GH_TOKEN=1 GH_TOKEN=... scripts/publish-wiki.sh
TEXT
  exit 1
fi
printf 'agenthub wiki published to %s\n' "$REMOTE"
