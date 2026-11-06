#!/usr/bin/env bash
# PreToolUse hook for Bash. Blocks commands that skip the project's checks or that
# only a contributor may run, whatever form the command takes. Asks the contributor
# before every pull-request creation or merge, and before the first commit or push
# after each of their messages: approving it opens a wave (review-wave.sh) that lets
# later commands made only of `cd`, `git add`, `git commit` and `git push` through
# until their next message. See AGENTS.md. Written for bash 3.2, the version macOS
# ships.
set -euo pipefail
set -f

if ! command -v jq >/dev/null 2>&1; then
  echo "Blocked: guard-bash.sh needs jq (see README.md)." >&2
  exit 2
fi
input=$(cat)
command=$(jq -r '.tool_input.command // ""' <<<"$input")
lower=$(printf '%s' "$command" | tr '[:upper:]' '[:lower:]')
ask_reason=""
gated=false
covered=true
case "$command" in
  *'>'* | *'<'*) covered=false ;;
esac

block() {
  echo "Blocked: $1. Fix the cause instead, or stop and report to the contributor (AGENTS.md)." >&2
  exit 2
}

no_verify="--no-verify skips the git hooks"
force_push="force pushes rewrite shared history"

case "$lower" in
  *--no-verify*) block "$no_verify" ;;
  *hookspath*) block "changing core.hooksPath skips the git hooks" ;;
  *lefthook_exclude=*) block "LEFTHOOK_EXCLUDE skips git hooks" ;;
  *"lefthook uninstall"*) block "uninstalling lefthook removes the git hooks" ;;
  *guard-approved*) block "only a contributor adds the guard-approved label (to write a file that mentions it, use the Edit or Write tool)" ;;
esac
lefthook_off='lefthook=["'"'"']?(0|false)'
if [[ "$lower" =~ $lefthook_off ]]; then
  block "disabling lefthook skips the git hooks"
fi

is_no_verify_prefix() {
  [[ ${#1} -ge 6 && "--no-verify" == "$1"* ]]
}

check_commit() {
  local word
  for word in "$@"; do
    if [[ "$word" =~ ^-[a-zA-Z]*n[a-zA-Z]*$ ]] || is_no_verify_prefix "$word"; then
      block "$no_verify"
    fi
  done
  gated=true
}

check_push() {
  local word
  for word in "$@"; do
    case "$word" in
      --force* | +*) block "$force_push" ;;
    esac
    if [[ ${#word} -ge 4 && "--force" == "$word"* ]] || [[ "$word" =~ ^-[a-zA-Z]*f[a-zA-Z]*$ ]]; then
      block "$force_push"
    fi
    if is_no_verify_prefix "$word"; then
      block "$no_verify"
    fi
  done
  gated=true
}

check_git() {
  local i=0 subcommand="" args=("$@")
  while [ "$i" -lt "${#args[@]}" ]; do
    case "${args[$i]}" in
      -C) i=$((i + 2)) ;;
      -c | --git-dir | --work-tree | --namespace | --config-env | --super-prefix)
        covered=false
        i=$((i + 2))
        ;;
      --no-pager) i=$((i + 1)) ;;
      -*)
        covered=false
        i=$((i + 1))
        ;;
      *)
        subcommand=${args[$i]}
        i=$((i + 1))
        break
        ;;
    esac
  done
  local rest=("${args[@]:$i}")
  case "$subcommand" in
    add) ;;
    commit) check_commit ${rest[@]+"${rest[@]}"} ;;
    push) check_push ${rest[@]+"${rest[@]}"} ;;
    *) covered=false ;;
  esac
}

check_gh() {
  case "${1:-} ${2:-}" in
    "pr create" | "pr merge")
      ask_reason="a contributor must approve opening or merging a pull request"
      return
      ;;
  esac
  [ "${1:-}" = "api" ] || return 0
  local word pulls=false writes=false
  for word in "$@"; do
    case "$word" in
      *pulls*) pulls=true ;;
      -X* | --method* | -f* | -F* | --field* | --raw-field* | --input*) writes=true ;;
    esac
  done
  if $pulls && $writes; then
    ask_reason="a contributor must approve changes to pull requests"
  fi
}

segments=$command
for separator in ';' '&' '|' '(' ')' '`' $'\n'; do
  segments=${segments//"$separator"/$'\n'}
done

while IFS= read -r segment; do
  words=()
  read -ra words <<<"$segment" || true
  i=0
  while [ "$i" -lt "${#words[@]}" ]; do
    word=${words[$i]}
    if [[ "$word" =~ ^[A-Za-z_][A-Za-z0-9_]*= ]]; then
      covered=false
      i=$((i + 1))
      continue
    fi
    case "$word" in
      command | env | exec | nohup | time | sudo | xargs)
        covered=false
        i=$((i + 1))
        ;;
      *) break ;;
    esac
  done
  [ "$i" -lt "${#words[@]}" ] || continue
  program=${words[$i]##*/}
  rest=("${words[@]:$((i + 1))}")
  case "$program" in
    cd) ;;
    git) check_git ${rest[@]+"${rest[@]}"} ;;
    gh)
      covered=false
      check_gh ${rest[@]+"${rest[@]}"}
      ;;
    *) covered=false ;;
  esac
done <<<"$segments"

session=$(jq -r '.session_id // ""' <<<"$input")
wave=""
if [[ "$session" =~ ^[A-Za-z0-9_-]+$ ]]; then
  tmp=${TMPDIR:-/tmp}
  wave="${tmp%/}/jukino-review-wave/$session"
fi

respond() {
  jq -n --arg decision "$1" --arg reason "Review gate: $2 (AGENTS.md)." \
    '{hookSpecificOutput: {hookEventName: "PreToolUse", permissionDecision: $decision, permissionDecisionReason: $reason}}'
}

if $gated && ! { [ -n "$wave" ] && [ -f "$wave/open" ]; }; then
  ask_reason=${ask_reason:-"a contributor approves the first commit or push after each of their messages"}
  if [ -n "$wave" ]; then
    mkdir -p "$wave"
    jq -r '.tool_use_id // ""' <<<"$input" >"$wave/pending"
  fi
fi
if [ -n "$ask_reason" ]; then
  respond ask "$ask_reason"
elif $gated && $covered; then
  respond allow "the contributor approved commits and pushes until their next message"
fi
exit 0
