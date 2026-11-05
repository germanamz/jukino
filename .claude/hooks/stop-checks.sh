#!/usr/bin/env bash
# Stop hook: when tracked Rust or TOML files have uncommitted changes, run the quick
# checks once before the agent finishes. A second stop in the same turn is let
# through, so the agent can stop and report a failure instead of working around it.
# `cargo xtask guard` is not run here: it must never block, and it runs in
# `cargo xtask ci`, pre-push and CI instead.
set -uo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "stop-checks.sh needs jq (see README.md); the quick checks did not run." >&2
  exit 1
fi
input=$(cat)
if [ "$(jq -r '.stop_hook_active // false' <<<"$input")" = "true" ]; then
  exit 0
fi
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
if [ -z "$(git status --porcelain -- '*.rs' '*.toml' 2>/dev/null)" ]; then
  exit 0
fi

failed=()
check() {
  local name=$1
  shift
  "$@" >/dev/null 2>&1 || failed+=("$name")
}
check fmt cargo fmt --all --check
check clippy cargo clippy --workspace --all-targets -- -D warnings
check comments cargo xtask comments
check arch cargo xtask arch
check nextest cargo nextest run --workspace

if [ ${#failed[@]} -gt 0 ]; then
  echo "Checks failed: ${failed[*]}. Fix them, or stop and tell the contributor which checks fail." >&2
  exit 2
fi
exit 0
