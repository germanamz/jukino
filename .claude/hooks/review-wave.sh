#!/usr/bin/env bash
# PostToolUse, PostToolUseFailure and UserPromptSubmit hook: tracks the review-gate
# wave that guard-bash.sh reads. When a commit or push the contributor approved
# has run, it opens the session's wave; the contributor's next message closes it.
# See AGENTS.md. Written for bash 3.2, the version macOS ships.
set -uo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "review-wave.sh needs jq (see README.md); the review-gate wave was not updated." >&2
  exit 1
fi
input=$(cat)
session=$(jq -r '.session_id // ""' <<<"$input")
[[ "$session" =~ ^[A-Za-z0-9_-]+$ ]] || exit 0
tmp=${TMPDIR:-/tmp}
wave="${tmp%/}/jukino-review-wave/$session"

case "$(jq -r '.hook_event_name // ""' <<<"$input")" in
  UserPromptSubmit) rm -rf "$wave" ;;
  PostToolUse | PostToolUseFailure)
    id=$(jq -r '.tool_use_id // ""' <<<"$input")
    if [ -n "$id" ] && [ -f "$wave/pending" ] && [ "$(cat "$wave/pending")" = "$id" ]; then
      mv "$wave/pending" "$wave/open"
    fi
    ;;
esac
exit 0
