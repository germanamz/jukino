@AGENTS.md

## Claude Code specifics

- Hooks in `.claude/settings.json`: `guard-bash.sh` blocks commands that skip checks or that only
  a contributor may run, and asks for permission before every commit, push, and pull-request
  creation or merge in any form (`git -C`, full paths, subshells); `after-edit.sh` formats an
  edited Rust file and runs the comments check; `stop-checks.sh` runs the quick checks once
  before a turn ends with uncommitted changes.
- `guard-bash.sh` matches some patterns anywhere in a shell command's text (the guard approval
  label, settings that disable git hooks), so write files that mention them with the Edit or
  Write tool.
- Path-scoped rules in `.claude/rules/` load when you touch matching files. Load the
  `placing-a-change` skill when planning any change.
- The Tusk mod (installed with `tusk claude install`) adds the Tusk MCP tools and a context
  digest at the start of each session.
