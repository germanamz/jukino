@AGENTS.md

## Claude Code specifics

- Hooks in `.claude/settings.json`: `guard-bash.sh` blocks commands that skip checks or that only
  a contributor may run, and asks for permission before every pull-request creation or merge
  and before the first commit or push after each contributor message, in any form (`git -C`,
  full paths, subshells); `review-wave.sh` records that approval as a wave that lets later
  commits and pushes through until the contributor's next message; `after-edit.sh` formats an
  edited Rust file and runs the comments check; `stop-checks.sh` runs the quick checks once
  before a turn ends with uncommitted changes.
- A wave lets through only commands made of `cd`, `git add`, `git commit` and `git push`, with
  no pipes, redirections or heredocs. Write the commit message to a file and pass it with
  `git commit -F <file>`.
- `guard-bash.sh` matches some patterns anywhere in a shell command's text (the guard approval
  label, settings that disable git hooks), so write files that mention them with the Edit or
  Write tool.
- Path-scoped rules in `.claude/rules/` load when you touch matching files. Load the
  `placing-a-change` skill when planning any change.
- The Tusk mod (installed with `tusk claude install`) adds the Tusk MCP tools and a context
  digest at the start of each session.
