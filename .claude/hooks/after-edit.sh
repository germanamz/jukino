#!/usr/bin/env bash
# PostToolUse hook for Edit and Write: formats an edited Rust file and runs the
# comments check on it; findings go back to the agent.
set -uo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "after-edit.sh needs jq (see README.md); the edited file was not checked." >&2
  exit 1
fi
file=$(jq -r '.tool_input.file_path // ""')
case "$file" in
  *.rs) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0

rustfmt --edition 2024 "$file" >/dev/null 2>&1 || true
findings=$(cargo xtask comments "$file" 2>/dev/null)
if [ $? -eq 1 ]; then
  echo "$findings" >&2
  exit 2
fi
exit 0
